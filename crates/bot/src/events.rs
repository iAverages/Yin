use std::sync::Arc;

use bot_core::command::{Command, CommandContext};
use bot_core::{BotState, Error};
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::Message;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::payload::incoming::GuildCreate;

pub async fn handle(
    state: Arc<BotState>,
    commands: Arc<[Command]>,
    shard_id: u32,
    event: Event,
) -> Result<(), Error> {
    feature_guilds::handle_event(&state.database, shard_id, &event).await?;

    match event {
        Event::Ready(ready) => tracing::info!(
            shard_id,
            bot_user = %ready.user.name,
            environment = %state.environment,
            "bot connected"
        ),
        Event::GuildCreate(guild) => {
            if let GuildCreate::Available(guild) = guild.as_ref() {
                state
                    .feature_flags
                    .identify_guild(guild.id.get(), &guild.name)?;
            }
        }
        Event::MessageCreate(message) => handle_message(&state, commands.len(), &message).await?,
        Event::MessageUpdate(message) => {
            feature_social::handle_message_update(&state, &message).await?;
        }
        Event::MessageDelete(message) => {
            feature_social::handle_message_delete(&state, message.channel_id, message.id).await?;
        }
        Event::InteractionCreate(interaction) => {
            handle_interaction(state, &commands, interaction.0).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn handle_message(
    state: &BotState,
    command_count: usize,
    message: &Message,
) -> Result<(), Error> {
    let prefix = feature_settings::active_prefix(&state.database, message.guild_id).await?;

    feature_admin::handle_message(state, message, &prefix, command_count).await?;
    feature_settings::handle_message(state, message, &prefix).await?;
    feature_social::handle_message(state, message).await
}

async fn handle_interaction(
    state: Arc<BotState>,
    commands: &[Command],
    mut interaction: Interaction,
) -> Result<(), Error> {
    match interaction.data.take() {
        Some(InteractionData::ApplicationCommand(data)) => {
            let Some(command) = commands
                .iter()
                .find(|command| command.definition.name == data.name)
            else {
                tracing::warn!(command = %data.name, "received unknown command");
                return Ok(());
            };

            let ctx = CommandContext::new(state, interaction, *data, command.ephemeral);
            run_command(command, &ctx).await;
        }
        Some(InteractionData::MessageComponent(data)) => {
            feature_settings::handle_interaction(&state, &interaction, &data).await?;
            feature_social::handle_interaction(&state, &interaction, &data).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn run_command(command: &Command, ctx: &CommandContext) {
    let trace_id = ctx.trace_id;
    let name = ctx.data.name.as_str();
    tracing::info!(%trace_id, command = name, "command started");

    match (command.run)(ctx).await {
        Ok(()) => tracing::info!(%trace_id, command = name, "command finished"),
        Err(error) => {
            tracing::error!(
                %trace_id,
                command = name,
                guild_id = ctx.guild_id().map(|id| id.get()),
                channel_id = ctx.interaction.channel.as_ref().map(|channel| channel.id.get()),
                user_id = ctx.author().id.get(),
                error = %format_error_chain(error.as_ref()),
                "command failed"
            );
            if let Err(error) = ctx
                .error("Something went wrong while running this command.")
                .await
            {
                tracing::error!(
                    %trace_id,
                    error = %format_error_chain(error.as_ref()),
                    "failed to send error embed"
                );
            }
        }
    }
}

pub fn format_error_chain(error: &(dyn std::error::Error + 'static)) -> String {
    let mut chain = error.to_string();
    let mut source = error.source();

    while let Some(error) = source {
        chain.push_str(": ");
        chain.push_str(&error.to_string());
        source = error.source();
    }

    chain
}
