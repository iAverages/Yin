use std::sync::Arc;
use std::time::Instant;

use bot_core::poise::{self, FrameworkError};
use bot_core::response::{self, Embed, EmbedKind};
use bot_core::serenity;
use bot_core::{BotState, Command, Environment, Error};
use database::GuildSettingsRepository;
use feature_settings::DEFAULT_PREFIX;

pub fn build(
    environment: Environment,
    dev_guild_id: Option<serenity::GuildId>,
    database: Arc<database::Database>,
    feature_flags: feature_flags::FeatureFlags,
) -> poise::Framework<BotState, Error> {
    poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: None,
                dynamic_prefix: Some(dynamic_prefix),
                mention_as_prefix: true,
                ..Default::default()
            },
            on_error: |error| Box::pin(on_error(error)),
            event_handler: |ctx, event, _, data| Box::pin(event_handler(ctx, event, data)),
            pre_command: |ctx| {
                Box::pin(async move {
                    bot_core::trace::initialize(ctx).await;
                })
            },
            post_command: |ctx| {
                Box::pin(async move {
                    let trace = bot_core::trace::current(ctx).await;
                    tracing::info!(
                        trace_id = %trace.trace_id,
                        command = %ctx.invocation_string(),
                        "command finished"
                    );
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                register_commands(ctx, framework, environment, dev_guild_id).await?;
                tracing::info!(
                    bot_user = %ready.user.name,
                    environment = %environment,
                    "bot connected"
                );

                let state = BotState {
                    started_at: Instant::now(),
                    environment,
                    database,
                    feature_flags,
                };
                start_moderation_workers(ctx, &state);
                Ok(state)
            })
        })
        .build()
}

fn commands() -> Vec<Command> {
    let mut commands = feature_admin::commands();
    commands.extend(feature_endfield::commands());
    commands.extend(feature_info::commands());
    commands.extend(feature_moderation::commands());
    commands.extend(feature_settings::commands());
    commands
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    data: &BotState,
) -> Result<(), Error> {
    feature_guilds::handle_event(&data.database, ctx.shard_id.0, event).await?;
    if let serenity::FullEvent::GuildCreate { guild, .. } = event {
        data.feature_flags
            .identify_guild(guild.id.get(), &guild.name)?;
    }
    if let serenity::FullEvent::Message { new_message } = event {
        feature_settings::handle_message(data, ctx, new_message).await?;
        feature_social::handle_message(data, ctx, new_message).await?;
    }
    if let serenity::FullEvent::MessageUpdate { event, .. } = event {
        feature_social::handle_message_update(data, ctx, event).await?;
    }
    if let serenity::FullEvent::MessageDelete {
        channel_id,
        deleted_message_id,
        ..
    } = event
    {
        feature_social::handle_message_delete(ctx, *channel_id, *deleted_message_id).await?;
    }
    if let serenity::FullEvent::GuildAuditLogEntryCreate { entry, guild_id } = event {
        feature_moderation::audit::process_audit_entry(&data.database, *guild_id, entry).await?;
    }
    Ok(())
}

fn start_moderation_workers(ctx: &serenity::Context, data: &BotState) {
    let http = ctx.http.clone();
    let cache = ctx.cache.clone();
    let database = data.database.clone();
    tokio::spawn(async move {
        let worker = format!("bot-{}", std::process::id());
        loop {
            if let Err(error) =
                feature_moderation::process_due_unlocks(&http, &database, &worker).await
            {
                tracing::error!(error = %error, "auto-unlock worker failed");
            }
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        }
    });

    let http = ctx.http.clone();
    let database = data.database.clone();
    tokio::spawn(async move {
        loop {
            let guilds = cache.guilds();
            if let Err(error) =
                feature_moderation::audit::reconcile_all_guilds(&http, &database, guilds).await
            {
                tracing::error!(error = %error, "audit-log reconciliation failed");
            }
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
}

fn dynamic_prefix(
    ctx: poise::PartialContext<'_, BotState, Error>,
) -> poise::BoxFuture<'_, Result<Option<String>, Error>> {
    Box::pin(async move {
        let Some(guild_id) = ctx.guild_id else {
            return Ok(Some(DEFAULT_PREFIX.to_owned()));
        };

        let repository = GuildSettingsRepository::new(&ctx.data.database);
        match repository.find_by_guild_id(guild_id.get()).await {
            Ok(settings) => Ok(Some(
                settings
                    .and_then(|settings| settings.command_prefix)
                    .unwrap_or_else(|| DEFAULT_PREFIX.to_owned()),
            )),
            Err(error) => {
                tracing::error!(
                    guild_id = %guild_id,
                    error = %format_error_chain(&error),
                    "failed to load guild prefix"
                );
                Ok(Some(DEFAULT_PREFIX.to_owned()))
            }
        }
    })
}

async fn register_commands(
    ctx: &serenity::Context,
    framework: &poise::Framework<BotState, Error>,
    environment: Environment,
    dev_guild_id: Option<serenity::GuildId>,
) -> Result<(), Error> {
    match environment {
        Environment::Development => {
            let guild_id = dev_guild_id.expect("development config requires a dev guild id");
            poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id)
                .await?;
            tracing::info!(guild_id = %guild_id, "registered guild commands");
        }
        Environment::Production => {
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            tracing::info!("registered global commands");
        }
    }

    Ok(())
}

async fn on_error(error: FrameworkError<'_, BotState, Error>) {
    if matches!(error, FrameworkError::UnknownCommand { .. }) {
        return;
    }
    let Some(ctx) = error.ctx() else {
        log_framework_error_without_context(&error);
        return;
    };

    if !matches!(error, FrameworkError::SubcommandRequired { .. }) {
        log_framework_error_with_context(&error, ctx).await;
    }

    let subcommands = ctx
        .command()
        .subcommands
        .iter()
        .map(|command| format!("`{}`", command.name))
        .collect::<Vec<_>>()
        .join(", ");
    let choose_subcommand = format!("Choose a subcommand: {subcommands}.");
    let title = match &error {
        FrameworkError::SubcommandRequired { .. } => choose_subcommand.as_str(),
        FrameworkError::ArgumentParse { .. } => "I could not parse that command argument.",
        FrameworkError::CooldownHit { .. } => "That command is on cooldown.",
        FrameworkError::MissingBotPermissions { .. } => {
            "I am missing permissions to run that command."
        }
        FrameworkError::MissingUserPermissions { .. } => {
            "You are missing permissions to run that command."
        }
        FrameworkError::NotAnOwner { .. } => "Only bot owners can use that command.",
        FrameworkError::GuildOnly { .. } => "This command can only be used in a server.",
        FrameworkError::DmOnly { .. } => "This command can only be used in DMs.",
        FrameworkError::NsfwOnly { .. } => "This command can only be used in an NSFW channel.",
        FrameworkError::CommandCheckFailed { error: None, .. } => {
            "You are missing permissions to run that command."
        }
        FrameworkError::CommandCheckFailed { error: Some(_), .. } => {
            "Something went wrong while checking command permissions."
        }
        FrameworkError::CommandPanic { .. } | FrameworkError::Command { .. } => {
            "Something went wrong while running this command."
        }
        _ => "Something went wrong while handling this command.",
    };

    if let Err(error) = response::send(ctx, Embed::new(EmbedKind::Error, title)).await {
        tracing::error!(
            error = %format_error_chain(error.as_ref()),
            "failed to send error embed"
        );
    }
}

async fn log_framework_error_with_context(
    error: &FrameworkError<'_, BotState, Error>,
    ctx: bot_core::Context<'_>,
) {
    let trace = bot_core::trace::current(ctx).await;
    let (message, source, detail): (_, Option<&Error>, Option<&str>) = match error {
        FrameworkError::Command { error, .. } => ("command failed", Some(error), None),
        FrameworkError::CommandCheckFailed {
            error: Some(error), ..
        } => ("command check failed", Some(error), None),
        FrameworkError::ArgumentParse { error, input, .. } => (
            "command argument parse failed",
            Some(error),
            input.as_deref(),
        ),
        FrameworkError::CommandPanic { payload, .. } => {
            ("command panicked", None, payload.as_deref())
        }
        _ => ("framework error", None, None),
    };
    let error = source.map_or_else(
        || error.to_string(),
        |error| format_error_chain(error.as_ref()),
    );
    tracing::error!(
        trace_id = %trace.trace_id,
        command = %ctx.invocation_string(),
        guild_id = ctx.guild_id().map(|id| id.get()),
        channel_id = ctx.channel_id().get(),
        user_id = ctx.author().id.get(),
        detail,
        error = %error,
        "{message}"
    );
}

fn log_framework_error_without_context(error: &FrameworkError<'_, BotState, Error>) {
    let (message, source) = match error {
        FrameworkError::Setup { error, .. } => ("framework setup failed", Some(error)),
        FrameworkError::EventHandler { error, event, .. } => {
            tracing::error!(
                event = event.snake_case_name(),
                error = %format_error_chain(error.as_ref()),
                "event handler failed"
            );
            return;
        }
        FrameworkError::DynamicPrefix { error, .. } => ("dynamic prefix failed", Some(error)),
        FrameworkError::NonCommandMessage { error, .. } => {
            ("non-command message handler failed", Some(error))
        }
        _ => ("framework error without command context", None),
    };
    let error = source.map_or_else(
        || error.to_string(),
        |error| format_error_chain(error.as_ref()),
    );
    tracing::error!(
        error = %error,
        "{message}"
    );
}

fn format_error_chain(error: &(dyn std::error::Error + 'static)) -> String {
    let mut chain = error.to_string();
    let mut source = error.source();

    while let Some(error) = source {
        chain.push_str(": ");
        chain.push_str(&error.to_string());
        source = error.source();
    }

    chain
}
