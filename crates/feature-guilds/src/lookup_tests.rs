use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use super::*;

fn discord_response(status: &str, body: &str) -> (String, thread::JoinHandle<Vec<String>>) {
    discord_responses(vec![(status, body.to_owned())])
}

fn discord_responses(pages: Vec<(&str, String)>) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/users/@me/guilds", listener.local_addr().unwrap());
    let responses: Vec<_> = pages.into_iter().map(|(status, body)| format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )).collect();
    let requests = thread::spawn(move || {
        responses
            .into_iter()
            .map(|response| {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                    .unwrap();
                let mut request = Vec::new();
                let mut buffer = [0; 1024];
                while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                    let count = stream.read(&mut buffer).unwrap();
                    assert_ne!(count, 0);
                    request.extend_from_slice(&buffer[..count]);
                }
                stream.write_all(response.as_bytes()).unwrap();
                String::from_utf8(request).unwrap()
            })
            .collect()
    });
    (url, requests)
}

#[tokio::test]
async fn write_lookup_requests_only_the_target_guild_with_fresh_credentials() {
    let (url, request) = discord_response(
        "200 OK",
        r#"[{"id":"9007199254740993","name":"Target","icon":null,"permissions":"32"}]"#,
    );
    let guild = fetch_user_guild_at(
        &reqwest::Client::new(),
        "fixture-token",
        9007199254740993,
        &url,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(guild.id, "9007199254740993");
    assert!(guild.can_manage());
    let request = request.join().unwrap().remove(0);
    assert!(request.starts_with("GET /users/@me/guilds?limit=1&after=9007199254740992 HTTP/1.1"));
    assert!(
        request
            .to_lowercase()
            .contains("authorization: bearer fixture-token\r\n")
    );
}

#[tokio::test]
async fn write_lookup_never_authorizes_the_next_guild_or_revoked_permissions() {
    for body in [
        "[]",
        r#"[{"id":"11","name":"Next guild","icon":null,"permissions":"8"}]"#,
        r#"[{"id":"10","name":"Revoked","icon":null,"permissions":"0"}]"#,
    ] {
        let (url, request) = discord_response("200 OK", body);
        let guild = fetch_user_guild_at(&reqwest::Client::new(), "fixture-token", 10, &url)
            .await
            .unwrap();
        assert!(guild.is_none_or(|guild| !guild.can_manage()));
        request.join().unwrap();
    }
}

#[tokio::test]
async fn write_lookup_fails_closed_on_discord_errors() {
    for status in [
        "401 Unauthorized",
        "403 Forbidden",
        "429 Too Many Requests",
        "500 Internal Server Error",
    ] {
        let (url, request) = discord_response(status, r#"{"message":"failed"}"#);
        assert!(
            fetch_user_guild_at(&reqwest::Client::new(), "fixture-token", 10, &url)
                .await
                .is_err()
        );
        request.join().unwrap();
    }
    for body in ["invalid JSON", r#"[{"id":"10"}]"#] {
        let (url, request) = discord_response("200 OK", body);
        assert!(
            fetch_user_guild_at(&reqwest::Client::new(), "fixture-token", 10, &url)
                .await
                .is_err()
        );
        request.join().unwrap();
    }
}

fn guild_page(start: u64, count: u64) -> String {
    serde_json::to_string(&(start..start + count).map(|id| serde_json::json!({
        "id": id.to_string(), "name": format!("Guild {id}"), "icon": null, "permissions": "32",
    })).collect::<Vec<_>>()).unwrap()
}

#[tokio::test]
async fn full_lookup_paginates_without_losing_snowflake_precision() {
    let (url, requests) = discord_responses(vec![
        ("200 OK", guild_page(9007199254740993, 200)),
        ("200 OK", guild_page(9007199254742000, 1)),
    ]);
    let guilds = fetch_user_guilds_at(&reqwest::Client::new(), "fixture-token", &url)
        .await
        .unwrap();
    assert_eq!(guilds.len(), 201);
    assert_eq!(guilds[0].id, "9007199254740993");
    assert_eq!(guilds[200].id, "9007199254742000");
    let requests = requests.join().unwrap();
    assert!(requests[0].starts_with("GET /users/@me/guilds?limit=200&after=0 HTTP/1.1"));
    assert!(
        requests[1].starts_with("GET /users/@me/guilds?limit=200&after=9007199254741192 HTTP/1.1")
    );
    assert!(requests.iter().all(|request| {
        request
            .to_lowercase()
            .contains("authorization: bearer fixture-token\r\n")
    }));
}

#[tokio::test]
async fn full_lookup_accepts_empty_membership() {
    let (url, requests) = discord_response("200 OK", "[]");
    let guilds = fetch_user_guilds_at(&reqwest::Client::new(), "fixture-token", &url)
        .await
        .unwrap();
    assert!(guilds.is_empty());
    requests.join().unwrap();
}

#[tokio::test]
async fn full_lookup_discards_partial_results_when_a_later_page_fails() {
    for status in [
        "401 Unauthorized",
        "403 Forbidden",
        "429 Too Many Requests",
        "500 Internal Server Error",
    ] {
        let (url, requests) = discord_responses(vec![
            ("200 OK", guild_page(1, 200)),
            (status, r#"{"message":"failed"}"#.to_owned()),
        ]);
        assert!(
            fetch_user_guilds_at(&reqwest::Client::new(), "fixture-token", &url)
                .await
                .is_err()
        );
        requests.join().unwrap();
    }
}

#[tokio::test]
async fn full_lookup_rejects_malformed_pages_and_nonadvancing_cursors() {
    for body in [
        "invalid JSON",
        r#"[{"id":123,"name":"Guild","icon":null,"permissions":"32"}]"#,
        r#"[{"id":"10"}]"#,
    ] {
        let (url, requests) = discord_response("200 OK", body);
        assert!(
            fetch_user_guilds_at(&reqwest::Client::new(), "fixture-token", &url)
                .await
                .is_err()
        );
        requests.join().unwrap();
    }
    let (url, requests) = discord_responses(vec![
        ("200 OK", guild_page(1, 200)),
        ("200 OK", guild_page(1, 200)),
    ]);
    assert!(matches!(
        fetch_user_guilds_at(&reqwest::Client::new(), "fixture-token", &url).await,
        Err(GuildError::InvalidPage)
    ));
    requests.join().unwrap();
}
