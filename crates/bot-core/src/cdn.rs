use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::user::User;
use twilight_model::util::ImageHash;

const CDN: &str = "https://cdn.discordapp.com";

pub fn avatar_url(user_id: Id<UserMarker>, avatar: Option<&ImageHash>) -> Option<String> {
    avatar.map(|hash| {
        let ext = if hash.is_animated() { "gif" } else { "webp" };
        format!("{CDN}/avatars/{user_id}/{hash}.{ext}?size=1024")
    })
}

pub fn face(user: &User) -> String {
    avatar_url(user.id, user.avatar.as_ref()).unwrap_or_else(|| {
        let index = if user.discriminator == 0 {
            (user.id.get() >> 22) % 6
        } else {
            u64::from(user.discriminator) % 5
        };

        format!("{CDN}/embed/avatars/{index}.png")
    })
}

pub fn guild_icon_url(guild_id: Id<GuildMarker>, icon: Option<&ImageHash>) -> Option<String> {
    icon.map(|hash| {
        let ext = if hash.is_animated() { "gif" } else { "webp" };
        format!("{CDN}/icons/{guild_id}/{hash}.{ext}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(id: u64, discriminator: u16, avatar: Option<&str>) -> User {
        serde_json::from_value(serde_json::json!({
            "id": id.to_string(),
            "username": "fixture",
            "discriminator": format!("{discriminator:04}"),
            "avatar": avatar,
        }))
        .unwrap()
    }

    #[test]
    fn face_uses_default_avatar_by_username_system() {
        let id = 80351110224678912;
        assert_eq!(
            face(&user(id, 0, None)),
            format!("{CDN}/embed/avatars/{}.png", (id >> 22) % 6)
        );
        assert_eq!(
            face(&user(id, 1337, None)),
            format!("{CDN}/embed/avatars/2.png")
        );
    }

    #[test]
    fn animated_avatars_use_gif() {
        let hash = "a_1269e74af4df7417b13759eae50c83dc";
        assert_eq!(
            face(&user(1, 0, Some(hash))),
            format!("{CDN}/avatars/1/{hash}.gif?size=1024")
        );
    }
}
