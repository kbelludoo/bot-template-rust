use serde_json::{json, Value};

/// Retorna o script JS de camuflagem anti-detecção para injetar via CDP (Page.addScriptToEvaluateOnNewDocument)
pub fn stealth_init_script() -> &'static str {
    r#"(() => {
  Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
  Object.defineProperty(navigator, 'languages', { get: () => ['pt-BR', 'pt', 'en-US', 'en'] });
  Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
  Object.defineProperty(navigator, 'hardwareConcurrency', { get: () => 8 });
  Object.defineProperty(navigator, 'deviceMemory', { get: () => 8 });
  window.chrome = window.chrome || { runtime: {} };
})();"#
}

/// Gera o hook de window.fetch e DummyWebSocket para capturar turnstileToken sem gastá-lo no IP do solver
pub fn turnstile_intercept_script(target_path_substring: &str) -> String {
    let target = if target_path_substring.trim().is_empty() {
        "/auth/world-ticket"
    } else {
        target_path_substring
    };

    format!(
        r#"(() => {{
  window.__INTERCEPTED_TURNSTILE_TOKEN = null;
  window.__INTERCEPTED_PAYLOAD = null;

  class DummyWebSocket {{
    constructor(url) {{
      this.url = url;
      this.readyState = 0;
    }}
    send() {{}}
    close() {{ this.readyState = 3; }}
    addEventListener() {{}}
    removeEventListener() {{}}
  }}
  window.WebSocket = DummyWebSocket;

  const origFetch = window.fetch.bind(window);
  window.fetch = async function(input, init) {{
    const url = typeof input === 'string' ? input : (input && input.url) || '';
    if (url.includes("{target}") && init && init.body) {{
      try {{
        const parsed = typeof init.body === 'string' ? JSON.parse(init.body) : init.body;
        window.__INTERCEPTED_PAYLOAD = parsed;
        const inner = (parsed && parsed["0"] && parsed["0"].json) ? parsed["0"].json : parsed;
        if (inner && inner.turnstileToken) {{
          window.__INTERCEPTED_TURNSTILE_TOKEN = inner.turnstileToken;
        }}
      }} catch (e) {{}}
      return new Response(JSON.stringify([{{ result: {{ data: {{ json: {{ intercepted: true }} }} }} }}]), {{
        status: 200,
        headers: {{ 'Content-Type': 'application/json' }}
      }});
    }}
    return origFetch(input, init);
  }};
}})();"#
    )
}

/// Constrói o objeto de telemetria "detector-2" com 14 sinais limpos e 0 pontos de suspeita
pub fn build_clean_anti_bot_laudo() -> Value {
    json!({
        "v": "detector-2",
        "sinais": {
            "wd": "nao",
            "cdc": "nao",
            "plg": "nao",
            "lng": "nao",
            "cr": "nao",
            "prm": "nao",
            "sw": "nao",
            "raf": "nao",
            "vis": "nao",
            "foco": "nao",
            "dims": "nao",
            "touch": "nao",
            "cores": "nao",
            "mem": "nao"
        },
        "pontos": 0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stealth_scripts() {
        assert!(stealth_init_script().contains("navigator"));
        let intercept = turnstile_intercept_script("/auth/token");
        assert!(intercept.contains("/auth/token"));
        assert!(intercept.contains("DummyWebSocket"));

        let laudo = build_clean_anti_bot_laudo();
        assert_eq!(laudo["v"], "detector-2");
        assert_eq!(laudo["pontos"], 0);
    }
}
