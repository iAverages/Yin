use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::command::Command as CommandDefinition;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::user::User;
use twilight_util::builder::interaction_response::ChannelMessageBuilder;
use uuid::Uuid;

use crate::response::{Embed, EmbedKind};
use crate::{BotState, Error};

pub type CommandFuture<'a> = Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;

pub struct Command {
    pub definition: CommandDefinition,
    pub run: fn(&CommandContext) -> CommandFuture<'_>,
    pub ephemeral: bool,
}

impl Command {
    pub fn new(
        definition: CommandDefinition,
        run: fn(&CommandContext) -> CommandFuture<'_>,
    ) -> Self {
        Self {
            definition,
            run,
            ephemeral: false,
        }
    }

    pub fn slash<T: CreateCommand>(run: fn(&CommandContext) -> CommandFuture<'_>) -> Self {
        Self::new(T::create_command().into(), run)
    }

    pub fn ephemeral(mut self) -> Self {
        self.ephemeral = true;
        self
    }
}

pub struct CommandContext {
    pub state: Arc<BotState>,
    pub interaction: Interaction,
    pub data: CommandData,
    pub trace_id: Uuid,
    ephemeral: AtomicBool,
    responded: AtomicBool,
}

impl CommandContext {
    pub fn new(
        state: Arc<BotState>,
        interaction: Interaction,
        data: CommandData,
        ephemeral: bool,
    ) -> Self {
        Self {
            state,
            interaction,
            data,
            trace_id: Uuid::now_v7(),
            ephemeral: AtomicBool::new(ephemeral),
            responded: AtomicBool::new(false),
        }
    }

    pub fn parse<T: CommandModel>(&self) -> Result<T, Error> {
        Ok(T::from_interaction(self.data.clone().into())?)
    }

    pub fn guild_id(&self) -> Option<Id<GuildMarker>> {
        self.interaction.guild_id
    }

    pub fn author(&self) -> &User {
        self.interaction
            .author()
            .expect("application command interactions always have an author")
    }

    pub fn set_ephemeral(&self) {
        self.ephemeral.store(true, Ordering::SeqCst);
    }

    pub fn mark_responded(&self) {
        self.responded.store(true, Ordering::SeqCst);
    }

    fn flags(&self) -> MessageFlags {
        if self.ephemeral.load(Ordering::SeqCst) {
            MessageFlags::EPHEMERAL
        } else {
            MessageFlags::empty()
        }
    }

    pub async fn defer(&self) -> Result<(), Error> {
        let response = InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: Some(InteractionResponseData {
                flags: Some(self.flags()),
                ..Default::default()
            }),
        };

        self.state
            .http
            .interaction(self.interaction.application_id)
            .create_response(self.interaction.id, &self.interaction.token, &response)
            .await?;

        self.responded.store(true, Ordering::SeqCst);

        Ok(())
    }

    pub async fn send(&self, embed: Embed) -> Result<(), Error> {
        let embeds = [embed.into()];
        let client = self.state.http.interaction(self.interaction.application_id);

        if self.responded.swap(true, Ordering::SeqCst) {
            client
                .create_followup(&self.interaction.token)
                .embeds(&embeds)
                .flags(self.flags())
                .await?;
        } else {
            let response = ChannelMessageBuilder::new()
                .embeds(embeds)
                .flags(self.flags())
                .build();
            client
                .create_response(self.interaction.id, &self.interaction.token, &response)
                .await?;
        }

        Ok(())
    }

    pub async fn error(&self, title: impl Into<String>) -> Result<(), Error> {
        self.send(Embed::new(EmbedKind::Error, title)).await
    }
}
