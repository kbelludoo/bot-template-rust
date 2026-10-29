use serde_json::{json, Value};

/// Monta o evento WebSocket de handshake de canal da Kick (wss://websockets.kick.com/viewer/v1/connect)
pub fn build_kick_handshake_payload(channel_id: &str) -> Value {
    json!({
        "type": "channel_handshake",
        "data": {
            "message": {
                "channelId": channel_id
            }
        }
    })
}

/// Monta o batimento de presença em live da Kick para acumular pontos/drops
pub fn build_kick_watch_livestream_payload(channel_id: &str, livestream_id: &str) -> Value {
    json!({
        "type": "user_event",
        "data": {
            "message": {
                "name": "tracking.user.watch.livestream",
                "channel_id": channel_id,
                "livestream_id": livestream_id
            }
        }
    })
}

/// Monta a mutação GraphQL oficial para resgatar baús de Channel Points na Twitch
pub fn build_twitch_claim_points_gql(channel_id: &str, claim_id: &str) -> Value {
    json!({
        "operationName": "ClaimCommunityPoints",
        "variables": {
            "input": {
                "channelID": channel_id,
                "claimID": claim_id
            }
        },
        "extensions": {
            "persistedQuery": {
                "version": 1,
                "sha256Hash": "46aaeebe02c99afdf4fc97c7c0cba964124bf6b0af229395f1f6d1feed05b3d0"
            }
        }
    })
}

/// Gera os comandos IRC para entrar em múltiplos canais na Twitch (irc.chat.twitch.tv:6697)
pub fn format_twitch_irc_join(channels: &[String]) -> Vec<String> {
    channels
        .iter()
        .map(|ch| ch.trim().trim_start_matches('#').to_lowercase())
        .filter(|ch| !ch.is_empty())
        .map(|ch| format!("JOIN #{ch}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twitch_irc_join() {
        let channels = vec!["#streamer1".to_string(), "Streamer2 ".to_string()];
        let joins = format_twitch_irc_join(&channels);
        assert_eq!(joins, vec!["JOIN #streamer1", "JOIN #streamer2"]);
    }

    #[test]
    fn test_kick_payloads() {
        let hs = build_kick_handshake_payload("12345");
        assert_eq!(hs["data"]["message"]["channelId"], "12345");

        let watch = build_kick_watch_livestream_payload("12345", "live_999");
        assert_eq!(watch["data"]["message"]["livestream_id"], "live_999");
    }
}
