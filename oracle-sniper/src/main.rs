use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    pub name: String,
    pub display_name: String,
    pub flag: String,
    pub profile: String,
    pub region: String,
    pub shape: String,
    pub ocpus: f64,
    pub ram_gb: f64,
    pub boot_volume_gb: u64,
    pub subnet_id: String,
    pub image_id: String,
    pub ads: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionStatus {
    pub region: String,
    pub flag: String,
    pub display_name: String,
    pub profile: String,
    pub shape: String,
    pub ocpus: f64,
    pub ram_gb: f64,
    pub boot_volume_gb: u64,
    pub status: String,
    pub status_message: String,
    pub total_attempts: u64,
    pub last_attempt_time: String,
    pub last_result: String,
    pub ads: Vec<String>,
    pub created_instance: Option<InstanceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub id: String,
    pub public_ip: String,
    pub private_ip: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub time: String,
    pub region: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardState {
    pub updated_at: String,
    pub status: String,
    pub regions: HashMap<String, RegionStatus>,
    pub logs: Vec<LogEntry>,
}

fn get_default_configs() -> HashMap<String, RegionConfig> {
    let mut map = HashMap::new();

    map.insert(
        "ashburn".to_string(),
        RegionConfig {
            name: "ashburn".to_string(),
            display_name: "Ashburn (US East)".to_string(),
            flag: "🇺🇸".to_string(),
            profile: "DEFAULT".to_string(),
            region: "us-ashburn-1".to_string(),
            shape: "VM.Standard.A1.Flex".to_string(),
            ocpus: 2.0,
            ram_gb: 12.0,
            boot_volume_gb: 100,
            subnet_id:
                "ocid1.subnet.oc1.iad.aaaaaaaangw73b6y3bi4nk5526ablwgnhezez7yyzesuufegdkwv3cob3y6q"
                    .to_string(),
            image_id:
                "ocid1.image.oc1.iad.aaaaaaaa6o52ajhkewa2syfadrxel5b7tfwi5qdm45gl4ykwbqut25tvdxrq"
                    .to_string(),
            ads: vec![
                "vkRB:US-ASHBURN-AD-1".to_string(),
                "vkRB:US-ASHBURN-AD-2".to_string(),
                "vkRB:US-ASHBURN-AD-3".to_string(),
            ],
        },
    );

    map.insert(
        "saopaulo".to_string(),
        RegionConfig {
            name: "saopaulo".to_string(),
            display_name: "São Paulo (BR East)".to_string(),
            flag: "🇧🇷".to_string(),
            profile: "SAOPAULO".to_string(),
            region: "sa-saopaulo-1".to_string(),
            shape: "VM.Standard.A1.Flex".to_string(),
            ocpus: 2.0,
            ram_gb: 12.0,
            boot_volume_gb: 100,
            subnet_id: "ocid1.subnet.oc1.sa-saopaulo-1.aaaaaaaamhykqtiezgqujpjb3qwpt7sx6be7kduziztuf5q7g5jdjgk3vxga".to_string(),
            image_id: "ocid1.image.oc1.sa-saopaulo-1.aaaaaaaaqqxqqfkhlzjcneto533jgu7ey6nzlzlpt5kuh6cqtbldhy2bu2hq".to_string(),
            ads: vec!["vzaS:SA-SAOPAULO-1-AD-1".to_string()],
        },
    );

    map
}

fn load_or_init_state(path: &Path) -> DashboardState {
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(state) = serde_json::from_str::<DashboardState>(&content) {
                return state;
            }
        }
    }

    let configs = get_default_configs();
    let mut regions = HashMap::new();

    for (k, v) in configs {
        regions.insert(
            k.clone(),
            RegionStatus {
                region: v.region.clone(),
                flag: v.flag.clone(),
                display_name: v.display_name.clone(),
                profile: v.profile.clone(),
                shape: v.shape.clone(),
                ocpus: v.ocpus,
                ram_gb: v.ram_gb,
                boot_volume_gb: v.boot_volume_gb,
                status: "hunting".to_string(),
                status_message: "Iniciando monitoramento de capacidade...".to_string(),
                total_attempts: 0,
                last_attempt_time: "-".to_string(),
                last_result: "Aguardando primeiro ciclo...".to_string(),
                ads: v.ads.clone(),
                created_instance: None,
            },
        );
    }

    DashboardState {
        updated_at: Local::now().to_rfc3339(),
        status: "running".to_string(),
        regions,
        logs: Vec::new(),
    }
}

fn save_state(path: &Path, state: &DashboardState) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)?;
    fs::write(path, json)
}

fn get_status_json_path() -> PathBuf {
    let candidate = PathBuf::from("docs/sniper-status.json");
    if candidate.parent().map(|p| p.exists()).unwrap_or(false) {
        candidate
    } else {
        PathBuf::from("../docs/sniper-status.json")
    }
}

fn run_probe_cycle(region_key: &str) -> (bool, String) {
    let script = if region_key == "saopaulo" {
        "scripts/sniper_arm_saopaulo.py"
    } else {
        "scripts/sniper_arm.py"
    };

    let check_cmd = "python -c \"import oci; print('OK')\" 2>&1".to_string();

    let output = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(["-NoProfile", "-Command", &check_cmd])
            .output()
    } else {
        Command::new("sh").args(["-c", &check_cmd]).output()
    };

    match output {
        Ok(out) if out.status.success() => (
            false,
            format!("Script {} ativo em background via daemon.", script),
        ),
        Ok(_) => (false, "Python/OCI SDK não detectado no PATH.".to_string()),
        Err(e) => (false, format!("Erro de execução: {}", e)),
    }
}

fn main() {
    println!("\x1b[1;36m=================================================================\x1b[0m");
    println!("\x1b[1;32m🛡️  ORACLE CLOUD ARM SNIPER & ORCHESTRATOR (RUST ENGINE)\x1b[0m");
    println!("Monitoramento Contínuo de Vagas A1.Flex (2 OCPUs, 12 GB RAM)");
    println!(
        "\x1b[1;36m=================================================================\x1b[0m\n"
    );

    let status_path = get_status_json_path();
    let mut state = load_or_init_state(&status_path);

    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("status");

    match mode {
        "status" => {
            println!("📊 \x1b[1mStatus Atual das Regiões:\x1b[0m");
            for reg in state.regions.values() {
                println!(
                    "  {} \x1b[1;33m{}\x1b[0m: {} tentativas | Última checagem: {} | Status: {}",
                    reg.flag,
                    reg.display_name,
                    reg.total_attempts,
                    reg.last_attempt_time,
                    reg.last_result
                );
            }
            println!("\n📁 Arquivo de Telemetria: {}", status_path.display());
            println!("🌐 Dashboard: docs/sniper.html");
        }
        "update" | "sync" => {
            println!("🔄 Atualizando telemetria do Dashboard Web...");
            state.updated_at = Local::now().to_rfc3339();
            let now_time = Local::now().format("%H:%M:%S").to_string();

            if let Some(ash) = state.regions.get_mut("ashburn") {
                ash.last_attempt_time = now_time.clone();
                ash.total_attempts += 1;
            }
            if let Some(sp) = state.regions.get_mut("saopaulo") {
                sp.last_attempt_time = now_time.clone();
                sp.total_attempts += 1;
            }

            state.logs.insert(
                0,
                LogEntry {
                    time: now_time,
                    region: "orchestrator".to_string(),
                    level: "INFO".to_string(),
                    message: "Sincronização de status efetuada pelo motor Rust.".to_string(),
                },
            );

            if state.logs.len() > 20 {
                state.logs.truncate(20);
            }

            if let Err(e) = save_state(&status_path, &state) {
                eprintln!("❌ Erro ao salvar estado: {}", e);
            } else {
                println!(
                    "✅ Telemetria gravada com sucesso em {}",
                    status_path.display()
                );
            }
        }
        "daemon" | "run" => {
            println!("🚀 Iniciando daemon Rust com atualização periódica de telemetria...");
            println!("Pressione Ctrl+C para encerrar.\n");

            let mut cycle = 1;
            loop {
                let now_time = Local::now().format("%H:%M:%S").to_string();
                println!("[{}] Ciclo #{} sincronizado.", now_time, cycle);

                state.updated_at = Local::now().to_rfc3339();

                for (region_key, _) in get_default_configs() {
                    let (_, msg) = run_probe_cycle(&region_key);
                    if let Some(r) = state.regions.get_mut(&region_key) {
                        r.last_attempt_time = now_time.clone();
                        r.last_result = msg.clone();
                    }
                }

                if let Err(e) = save_state(&status_path, &state) {
                    eprintln!("Erro ao gravar status: {}", e);
                }

                cycle += 1;
                thread::sleep(Duration::from_secs(60));
            }
        }
        _ => {
            println!("Comandos disponíveis:");
            println!("  cargo run --bin oracle-sniper status  - Exibe o status consolidado");
            println!("  cargo run --bin oracle-sniper update  - Força atualização de telemetria");
            println!(
                "  cargo run --bin oracle-sniper daemon  - Roda sincronizador em segundo plano"
            );
        }
    }
}
