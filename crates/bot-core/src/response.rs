use twilight_model::channel::message::embed::{EmbedImage, EmbedThumbnail};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

const INFO_COLOR: u32 = 0x805ad5;
const SUCCESS_COLOR: u32 = 0x38a169;
const WARNING_COLOR: u32 = 0xdd6b20;
const ERROR_COLOR: u32 = 0xe53e3e;

#[derive(Debug, Clone, Copy)]
pub enum EmbedKind {
    Info,
    Success,
    Warning,
    Error,
}

impl EmbedKind {
    fn color(self) -> u32 {
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
    image: Option<String>,
    thumbnail: Option<String>,
}

impl Embed {
    pub fn new(kind: EmbedKind, title: impl Into<String>) -> Self {
        Self {
            kind,
            title: title.into(),
            description: None,
            fields: Vec::new(),
            image: None,
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

    pub fn image(mut self, url: impl Into<String>) -> Self {
        self.image = Some(url.into());
        self
    }
}

impl From<Embed> for twilight_model::channel::message::Embed {
    fn from(embed: Embed) -> Self {
        let mut builder = EmbedBuilder::new()
            .title(embed.title)
            .color(embed.kind.color());

        if let Some(description) = embed.description {
            builder = builder.description(description);
        }

        for (name, value, inline) in embed.fields {
            let field = EmbedFieldBuilder::new(name, value);
            builder = builder.field(if inline { field.inline() } else { field });
        }

        let mut built = builder.build();
        built.thumbnail = embed.thumbnail.map(|url| EmbedThumbnail {
            height: None,
            proxy_url: None,
            url,
            width: None,
        });

        built.image = embed.image.map(|url| EmbedImage {
            content_type: None,
            flags: None,
            height: None,
            placeholder: None,
            placeholder_version: None,
            proxy_url: None,
            url,
            width: None,
        });
        built
    }
}
