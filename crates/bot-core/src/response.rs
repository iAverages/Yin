use crate::serenity::{self, CreateEmbed};
use crate::{Context, Error};

const INFO_COLOR: serenity::Colour = serenity::Colour::from_rgb(128, 90, 213);
const SUCCESS_COLOR: serenity::Colour = serenity::Colour::from_rgb(56, 161, 105);
const WARNING_COLOR: serenity::Colour = serenity::Colour::from_rgb(221, 107, 32);
const ERROR_COLOR: serenity::Colour = serenity::Colour::from_rgb(229, 62, 62);

#[derive(Debug, Clone, Copy)]
pub enum EmbedKind {
    Info,
    Success,
    Warning,
    Error,
}

impl EmbedKind {
    fn color(self) -> serenity::Colour {
        match self {
            Self::Info => INFO_COLOR,
            Self::Success => SUCCESS_COLOR,
            Self::Warning => WARNING_COLOR,
            Self::Error => ERROR_COLOR,
        }
    }
}

pub struct Embed {
    kind: EmbedKind,
    title: String,
    description: Option<String>,
    fields: Vec<(String, String, bool)>,
    thumbnail: Option<String>,
}

impl Embed {
    pub fn new(kind: EmbedKind, title: impl Into<String>) -> Self {
        Self {
            kind,
            title: title.into(),
            description: None,
            fields: Vec::new(),
            thumbnail: None,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn field(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        inline: bool,
    ) -> Self {
        self.fields.push((name.into(), value.into(), inline));
        self
    }

    pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
        self.thumbnail = Some(url.into());
        self
    }
}

pub async fn send(ctx: Context<'_>, embed: Embed) -> Result<(), Error> {
    let mut create = CreateEmbed::new()
        .title(embed.title)
        .color(embed.kind.color());

    if let Some(description) = embed.description {
        create = create.description(description);
    }

    if let Some(thumbnail) = embed.thumbnail {
        create = create.thumbnail(thumbnail);
    }

    for (name, value, inline) in embed.fields {
        create = create.field(name, value, inline);
    }

    ctx.send(crate::poise::CreateReply::default().embed(create))
        .await?;
    Ok(())
}

pub async fn info(ctx: Context<'_>, title: impl Into<String>) -> Result<(), Error> {
    send(ctx, Embed::new(EmbedKind::Info, title)).await
}

pub async fn error(ctx: Context<'_>, title: impl Into<String>) -> Result<(), Error> {
    send(ctx, Embed::new(EmbedKind::Error, title)).await
}
