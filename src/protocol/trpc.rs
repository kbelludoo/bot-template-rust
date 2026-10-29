use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};

/// Formata o payload no formato tRPC v10 Batch `{"0":{"json":...}}` (usado com `?batch=1`)
pub fn format_trpc_batch_input<T: Serialize>(input: &T) -> Result<String, serde_json::Error> {
    let envelope = json!({
        "0": {
            "json": input
        }
    });
    serde_json::to_string(&envelope)
}

/// Desenvelopa a resposta tRPC v10 `[{"result":{"data":{"json":...}}}]` para o tipo alvo
pub fn unwrap_trpc_batch_response<T: DeserializeOwned>(body: &str) -> Result<T, String> {
    let parsed: Value =
        serde_json::from_str(body).map_err(|e| format!("resposta tRPC batch malformada: {e}"))?;

    let arr = parsed
        .as_array()
        .ok_or_else(|| "resposta tRPC batch não é array".to_string())?;

    if arr.is_empty() {
        return Err("resposta tRPC batch vazia".to_string());
    }

    let first = &arr[0];

    // Verificar se houve erro
    if let Some(err) = first.get("error") {
        let code = err
            .get("json")
            .and_then(|j| j.get("code"))
            .and_then(|c| c.as_i64())
            .unwrap_or(500);
        let msg = err
            .get("json")
            .and_then(|j| j.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("erro desconhecido");
        return Err(format!("erro tRPC ({code}): {msg}"));
    }

    let json_data = first
        .get("result")
        .and_then(|r| r.get("data"))
        .and_then(|d| d.get("json"))
        .ok_or_else(|| "payload tRPC sem data.json".to_string())?;

    serde_json::from_value(json_data.clone())
        .map_err(|e| format!("falha ao desserializar data.json: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Dummy {
        message: String,
    }

    #[test]
    fn test_trpc_roundtrip() {
        let input = Dummy {
            message: "hello".to_string(),
        };
        let formatted = format_trpc_batch_input(&input).unwrap();
        assert!(formatted.contains("\"0\":{\"json\":{\"message\":\"hello\"}}"));

        let response_json = r#"[{"result":{"data":{"json":{"message":"world"}}}}]"#;
        let res: Dummy = unwrap_trpc_batch_response(response_json).unwrap();
        assert_eq!(res.message, "world");
    }

    #[test]
    fn test_trpc_error() {
        let response_err = r#"[{"error":{"json":{"code":-32600,"message":"Invalid request"}}}]"#;
        let res: Result<Dummy, String> = unwrap_trpc_batch_response(response_err);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Invalid request"));
    }
}
