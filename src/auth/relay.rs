use std::fs;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::info;

static LAST_SOLVER_KICK: Mutex<Option<Instant>> = Mutex::new(None);

/// Lê o arquivo de log do túnel (ex: cloudflared.log) e retorna a URL pública ativa do trycloudflare.com
pub fn extrair_ultima_tunnel_url(log_path: &str) -> String {
    let data = match fs::read_to_string(log_path) {
        Ok(d) => d,
        Err(_) => return String::new(),
    };

    let mut last_match = String::new();
    for line in data.lines() {
        if let Some(idx) = line.find("https://") {
            let sub = &line[idx..];
            if let Some(end_idx) = sub.find(".trycloudflare.com") {
                let url = &sub[..end_idx + ".trycloudflare.com".len()];
                last_match = url.to_string();
            }
        }
    }
    last_match
}

/// Dispara o workflow ticket-relay no GitHub Actions em mode=push
pub fn disparar_solver_actions(
    repo: &str,
    workflow_file: &str,
    tunnel_log_path: &str,
    log_id: &str,
    cooldown: Duration,
) -> Result<(), String> {
    let mut lock = LAST_SOLVER_KICK.lock().unwrap();
    let cd = if cooldown.is_zero() {
        Duration::from_secs(45)
    } else {
        cooldown
    };

    if let Some(last) = *lock {
        if last.elapsed() < cd {
            let rem = cd - last.elapsed();
            return Err(format!("cooldown ativo ({}s restante)", rem.as_secs()));
        }
    }

    let tunnel = extrair_ultima_tunnel_url(tunnel_log_path);
    if tunnel.is_empty() {
        return Err(format!(
            "[{log_id}] nenhuma tunnel URL encontrada em {tunnel_log_path}"
        ));
    }

    *lock = Some(Instant::now());

    // Dispara o runner no GitHub via CLI nativo `gh`
    let output = Command::new("gh")
        .args([
            "workflow",
            "run",
            workflow_file,
            "-R",
            repo,
            "-f",
            "mode=push",
            "-f",
            &format!("push_url={tunnel}"),
        ])
        .output()
        .map_err(|e| format!("[{log_id}] falha ao executar gh: {e}"))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("[{log_id}] gh workflow run falhou: {err}"));
    }

    info!("[{log_id}] ✔ Solver GitHub Actions disparado com sucesso: {workflow_file} (push em {tunnel})");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extrair_tunnel_url() {
        let temp_dir = std::env::temp_dir();
        let log_file = temp_dir.join("test_tunnel.log");
        let content = "Starting tunnel...\nINF Your quick Tunnel has been created! Visit it at (https://abc-123.trycloudflare.com)\nConnected";
        std::fs::write(&log_file, content).unwrap();

        let extracted = extrair_ultima_tunnel_url(log_file.to_str().unwrap());
        assert_eq!(extracted, "https://abc-123.trycloudflare.com");

        let _ = std::fs::remove_file(log_file);
    }
}
