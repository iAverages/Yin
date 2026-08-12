use std::sync::Arc;
use std::time::Instant;

use bot_core::poise::{self, FrameworkError};
use bot_core::response::{self, Embed, EmbedKind};
use bot_core::serenity;
use bot_core::{BotState, Command, Environment, Error};
use database::GuildSettingsRepository;

const DEFAULT_PREFIX: &str = "!";

pub fn build(
    environment: Environment,
    dev_guild_id: Option<serenity::GuildId>,
    database: Arc<database::Database>,
    auth_service_url: String,
    auth_internal_token: Option<String>,
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

                Ok(BotState {
                    started_at: Instant::now(),
                    environment,
                    database,
                    auth_service_url,
                    auth_internal_token,
                })
            })
        })
        .build()
}

fn commands() -> Vec<Command> {
    let mut commands = feature_admin::commands();
    commands.extend(feature_endfield::commands());
    commands.extend(feature_info::commands());
    commands.extend(feature_settings::commands());
    commands
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
                    error = %error,
                    error_chain = %format_error_chain(&error),
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
    let Some(ctx) = error.ctx() else {
        log_framework_error_without_context(&error);
        return;
    };

    log_framework_error_with_context(&error, ctx).await;

    let title = match &error {
        FrameworkError::SubcommandRequired { .. } => "Choose an info subcommand.",
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
            error = %error,
            error_chain = %format_error_chain(error.as_ref()),
            "failed to send error embed"
        );
    }
}

async fn log_framework_error_with_context(
    error: &FrameworkError<'_, BotState, Error>,
    ctx: bot_core::Context<'_>,
) {
    let trace = bot_core::trace::current(ctx).await;
    let command = ctx.invocation_string();
    let guild_id = ctx.guild_id().map(|id| id.get());
    let channel_id = ctx.channel_id().get();
    let user_id = ctx.author().id.get();

    match error {
        FrameworkError::Command { error, .. } => {
            tracing::error!(
                trace_id = %trace.trace_id,
                command = %command,
                guild_id,
                channel_id,
                user_id,
                error = %error,
                error_chain = %format_error_chain(error.as_ref()),
                "command failed"
            );
        }
        FrameworkError::CommandCheckFailed {
            error: Some(error), ..
        } => {
            tracing::error!(
                trace_id = %trace.trace_id,
                command = %command,
                guild_id,
                channel_id,
                user_id,
                error = %error,
                error_chain = %format_error_chain(error.as_ref()),
                "command check failed"
            );
        }
        FrameworkError::ArgumentParse { error, input, .. } => {
            tracing::error!(
                trace_id = %trace.trace_id,
                command = %command,
                guild_id,
                channel_id,
                user_id,
                input,
                error = %error,
                error_chain = %format_error_chain(error.as_ref()),
                "command argument parse failed"
            );
        }
        FrameworkError::CommandPanic { payload, .. } => {
            tracing::error!(
                trace_id = %trace.trace_id,
                command = %command,
                guild_id,
                channel_id,
                user_id,
                payload,
                "command panicked"
            );
        }
        _ => {
            tracing::error!(
                trace_id = %trace.trace_id,
                command = %command,
                guild_id,
                channel_id,
                user_id,
                error = %error,
                "framework error"
            );
        }
    }
}

fn log_framework_error_without_context(error: &FrameworkError<'_, BotState, Error>) {
    match error {
        FrameworkError::Setup { error, .. } => {
            tracing::error!(
                error = %error,
                error_chain = %format_error_chain(error.as_ref()),
                "framework setup failed"
            );
        }
        FrameworkError::EventHandler { error, event, .. } => {
            tracing::error!(
                event = event.snake_case_name(),
                error = %error,
                error_chain = %format_error_chain(error.as_ref()),
                "event handler failed"
            );
        }
        FrameworkError::DynamicPrefix { error, .. } => {
            tracing::error!(
                error = %error,
                error_chain = %format_error_chain(error.as_ref()),
                "dynamic prefix failed"
            );
        }
        FrameworkError::NonCommandMessage { error, .. } => {
            tracing::error!(
                error = %error,
                error_chain = %format_error_chain(error.as_ref()),
                "non-command message handler failed"
            );
        }
        _ => {
            tracing::error!(error = %error, "framework error without command context");
        }
    }
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
