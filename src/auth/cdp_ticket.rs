use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketResult {
    pub ticket: String,
    pub ws_url: String,
}

/// Localiza o executável do Chrome/Chromium no sistema operacional
pub fn find_browser_path() -> Result<String, String> {
    if let Ok(custom) = env::var("BROWSER_PATH") {
        if Path::new(&custom).exists() {
            return Ok(custom);
        }
    }

    #[cfg(target_os = "windows")]
    let candidates = [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string(),
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe".to_string(),
        format!(
            r"{}\Google\Chrome\Application\chrome.exe",
            env::var("LOCALAPPDATA").unwrap_or_default()
        ),
        r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe".to_string(),
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe".to_string(),
    ];

    #[cfg(not(target_os = "windows"))]
    let candidates = [
        "/usr/bin/google-chrome".to_string(),
        "/usr/bin/google-chrome-stable".to_string(),
        "/usr/bin/chromium".to_string(),
        "/usr/bin/chromium-browser".to_string(),
    ];

    for c in candidates.iter() {
        if !c.is_empty() && Path::new(c).exists() {
            return Ok(c.clone());
        }
    }

    Err("nenhum navegador Chrome/Chromium encontrado no sistema".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_browser_path() {
        // Testa se a função executa sem pânico e retorna Result
        let _ = find_browser_path();
    }
}
