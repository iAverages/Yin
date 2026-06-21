pub mod config;
pub mod error;
pub mod permissions;
pub mod response;
pub mod state;
pub mod time;
pub mod trace;

pub use poise;
pub use poise::serenity_prelude as serenity;

pub use config::Environment;
pub use error::Error;
pub use state::BotState;

pub type Context<'a> = poise::Context<'a, BotState, Error>;
pub type Command = poise::Command<BotState, Error>;
