use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::Utc;
use rand::Rng;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::Pkcs1v15Sign;
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    pub name: String,
    pub display_name: String,
    pub flag: String,
    pub profile: String,
    pub region: String,
    pub tenancy_id: String,
    pub user_id: String,
    pub fingerprint: String,
    pub key_path: String,
    pub shape: String,
    pub ocpus: f64,
    pub ram_gb: f64,
    pub boot_volume_gb: u64,
    pub subnet_id: String,
    pub image_id: String,
    pub ads: Vec<String>,
    pub ssh_key: String,
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

pub struct OciSigner {
    pub tenancy_id: String,
    pub user_id: String,
    pub fingerprint: String,
    pub private_key: RsaPrivateKey,
}

impl OciSigner {
    pub fn load_from_file(path: &str, tenancy: &str, user: &str, fp: &str) -> Result<Self, String> {
        let pem_str = fs::read_to_string(path)
            .map_err(|e| format!("Falha ao ler chave privada em {}: {}", path, e))?;

        let clean_pem = if let Some(start) = pem_str.find("-----BEGIN") {
            if let Some(end) = pem_str.find("-----END") {
                if let Some(end_line) = pem_str[end..].find('\n') {
                    pem_str[start..end + end_line].trim().replace("\r\n", "\n")
                } else {
                    pem_str[start..].trim().replace("\r\n", "\n")
                }
            } else {
                pem_str.replace("\r\n", "\n")
            }
        } else {
            pem_str.replace("\r\n", "\n")
        };

        let key = if let Ok(k) = RsaPrivateKey::from_pkcs8_pem(&clean_pem) {
            k
        } else if let Ok(k) = RsaPrivateKey::from_pkcs1_pem(&clean_pem) {
            k
        } else {
            // Tenta decodificar a versao bruta
            let trimmed = pem_str.trim().replace("\r\n", "\n");
            if let Ok(k) = RsaPrivateKey::from_pkcs8_pem(&trimmed) {
                k
            } else if let Ok(k) = RsaPrivateKey::from_pkcs1_pem(&trimmed) {
                k
            } else {
                return Err("Formato de chave PEM invalido (nem PKCS#1 nem PKCS#8)".to_string());
            }
        };

        Ok(Self {
            tenancy_id: tenancy.to_string(),
            user_id: user.to_string(),
            fingerprint: fp.to_string(),
            private_key: key,
        })
    }

    pub fn build_auth_header(
        &self,
        method: &str,
        path_query: &str,
        host: &str,
        date: &str,
        body: Option<&[u8]>,
    ) -> Result<(String, Option<String>, Option<usize>), String> {
        let target_line = format!("(request-target): {} {}", method.to_lowercase(), path_query);
        let date_line = format!("date: {}", date);
        let host_line = format!("host: {}", host);

        let (headers_list, string_to_sign, sha256_b64, cl) = if let Some(b) = body {
            let mut hasher = Sha256::new();
            hasher.update(b);
            let hash_b64 = BASE64.encode(hasher.finalize());

            let cl = b.len();
            let headers = "(request-target) date host content-length content-type x-content-sha256";
            let s = format!(
                "{}\n{}\n{}\ncontent-length: {}\ncontent-type: application/json\nx-content-sha256: {}",
                target_line, date_line, host_line, cl, hash_b64
            );
            (headers, s, Some(hash_b64), Some(cl))
        } else {
            let headers = "(request-target) date host";
            let s = format!("{}\n{}\n{}", target_line, date_line, host_line);
            (headers, s, None, None)
        };

        let mut hasher = Sha256::new();
        hasher.update(string_to_sign.as_bytes());
        let hashed = hasher.finalize();

        // PKCS#1 v1.5 DigestInfo prefix for SHA-256 (RFC 3447)
        const SHA256_PREFIX: &[u8] = &[
            0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02,
            0x01, 0x05, 0x00, 0x04, 0x20,
        ];
        let mut digest_info = Vec::with_capacity(SHA256_PREFIX.len() + hashed.len());
        digest_info.extend_from_slice(SHA256_PREFIX);
        digest_info.extend_from_slice(&hashed);

        let signature = self
            .private_key
            .sign(Pkcs1v15Sign::new_unprefixed(), &digest_info)
            .map_err(|e| format!("Sign error: {}", e))?;

        let sig_b64 = BASE64.encode(signature);

        let key_id = format!("{}/{}/{}", self.tenancy_id, self.user_id, self.fingerprint);
        let auth_val = format!(
            "Signature version=\"1\",keyId=\"{}\",algorithm=\"rsa-sha256\",headers=\"{}\",signature=\"{}\"",
            key_id, headers_list, sig_b64
        );

        Ok((auth_val, sha256_b64, cl))
    }
}

fn get_status_json_path() -> PathBuf {
    let candidate = PathBuf::from("docs/sniper-status.json");
    if candidate.parent().map(|p| p.exists()).unwrap_or(false) {
        candidate
    } else {
        PathBuf::from("../docs/sniper-status.json")
    }
}

fn update_status_json(
    reg_name: &str,
    attempt: u64,
    last_time: &str,
    msg: &str,
    created: Option<InstanceInfo>,
) {
    let path = get_status_json_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(mut state) = serde_json::from_str::<DashboardState>(&content) {
            state.updated_at = Utc::now().to_rfc3339();
            if let Some(r) = state.regions.get_mut(reg_name) {
                r.total_attempts = attempt;
                r.last_attempt_time = last_time.to_string();
                r.last_result = msg.to_string();
                if let Some(ref inst) = created {
                    r.status = "active".to_string();
                    r.created_instance = Some(inst.clone());
                }
            }
            state.logs.insert(
                0,
                LogEntry {
                    time: last_time.to_string(),
                    region: reg_name.to_string(),
                    level: if created.is_some() {
                        "SUCCESS".to_string()
                    } else {
                        "INFO".to_string()
                    },
                    message: msg.to_string(),
                },
            );
            if state.logs.len() > 25 {
                state.logs.truncate(25);
            }
            if let Ok(json) = serde_json::to_string_pretty(&state) {
                let _ = fs::write(&path, json);
            }
        }
    }
}

#[derive(Serialize)]
struct LaunchPayload<'a> {
    #[serde(rename = "compartmentId")]
    compartment_id: &'a str,
    #[serde(rename = "availabilityDomain")]
    availability_domain: &'a str,
    #[serde(rename = "displayName")]
    display_name: &'a str,
    shape: &'a str,
    #[serde(rename = "shapeConfig")]
    shape_config: ShapeConfig,
    #[serde(rename = "sourceDetails")]
    source_details: SourceDetails<'a>,
    #[serde(rename = "createVnicDetails")]
    create_vnic_details: VnicDetails<'a>,
    metadata: HashMap<&'a str, &'a str>,
}

#[derive(Serialize)]
struct ShapeConfig {
    ocpus: f64,
    #[serde(rename = "memoryInGBs")]
    memory_in_gbs: f64,
}

#[derive(Serialize)]
struct SourceDetails<'a> {
    #[serde(rename = "sourceType")]
    source_type: &'a str,
    #[serde(rename = "imageId")]
    image_id: &'a str,
    #[serde(rename = "bootVolumeSizeInGBs")]
    boot_volume_size_in_gbs: u64,
}

#[derive(Serialize)]
struct VnicDetails<'a> {
    #[serde(rename = "subnetId")]
    subnet_id: &'a str,
    #[serde(rename = "assignPublicIp")]
    assign_public_ip: bool,
    #[serde(rename = "displayName")]
    display_name: &'a str,
}

#[tokio::main]
async fn main() {
    println!("\x1b[1;36m=================================================================\x1b[0m");
    println!("\x1b[1;32m🦀 ORACLE ARM SNIPER — 100% RUST ZERO-GC ENGINE\x1b[0m");
    println!("Alvo: Ubuntu 26.04 Minimal ARM (2 OCPUs, 12 GB RAM, 100 GB NVMe)");
    println!("Consumo Estimado de Memoria: ~2 MB RAM | Zero Python Runtime");
    println!(
        "\x1b[1;36m=================================================================\x1b[0m\n"
    );

    let ssh_key_sp = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDSx8Loag4kbo5jj2p4+REZzO6TanBxPDTKyCavdIQKKcIfoyVJgS7cnNAIfEZdV5S5jbH7xmOXGz5B3EXJngScCAHawyve2MPyR6C1z/UNduhsR4PvHlyWldvVLvT1lNXjGIwVGh21maVHPdFK2YHA1up7Zx3ttfyqSzxQX19/gyx805DLTm5EA7MF1ja6p8J4N6JACqnfTMVTkDwKUNNSxOXdCuYh3IZkDgJ9QdGhHm8D7OwmvyZdzgrZ2F5q8upP1F1b3F80gh9XMJaLJFK5xUFzWJykudFiqz5sRUUVU/LlUjsEKQjPFtXtlsbL1XKv1tDbhHO8wouTI5EQxdT5 ssh-key-2026-10-09";

    // Configuracao de Sao Paulo
    let sp_config = RegionConfig {
        name: "saopaulo".to_string(),
        display_name: "São Paulo (BR East)".to_string(),
        flag: "🇧🇷".to_string(),
        profile: "SAOPAULO".to_string(),
        region: "sa-saopaulo-1".to_string(),
        tenancy_id: "ocid1.tenancy.oc1..aaaaaaaagqnk2dmiakcslalb62zdjaqktp74g5ktvyqamrizu2cimxoeij5q".to_string(),
        user_id: "ocid1.user.oc1..aaaaaaaavw35prmrdzh3r4tyomphzni7jb3ornnfninipwwgzkgdsgzqfpja".to_string(),
        fingerprint: "32:10:91:56:8a:0d:23:08:bb:69:41:78:dd:74:24:64".to_string(),
        key_path: "C:\\Users\\kbell\\.oci\\saopaulo_key.pem".to_string(),
        shape: "VM.Standard.A1.Flex".to_string(),
        ocpus: 2.0,
        ram_gb: 12.0,
        boot_volume_gb: 100,
        subnet_id: "ocid1.subnet.oc1.sa-saopaulo-1.aaaaaaaamhykqtiezgqujpjb3qwpt7sx6be7kduziztuf5q7g5jdjgk3vxga".to_string(),
        image_id: "ocid1.image.oc1.sa-saopaulo-1.aaaaaaaamgy43642gy2ta7pdh6etf6ye5kspe5u5bvj4hxvhrgjqwi3lohkq".to_string(),
        ads: vec!["vzaS:SA-SAOPAULO-1-AD-1".to_string()],
        ssh_key: ssh_key_sp.to_string(),
    };

    let signer = match OciSigner::load_from_file(
        &sp_config.key_path,
        &sp_config.tenancy_id,
        &sp_config.user_id,
        &sp_config.fingerprint,
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ Erro ao inicializar Signer OCI em Rust: {}", e);
            return;
        }
    };

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();

    let mut attempt = 1u64;
    let mut rng = rand::thread_rng();

    loop {
        let now_time = chrono::Local::now().format("%H:%M:%S").to_string();

        let mut metadata = HashMap::new();
        metadata.insert("ssh_authorized_keys", sp_config.ssh_key.as_str());

        let payload = LaunchPayload {
            compartment_id: &sp_config.tenancy_id,
            availability_domain: &sp_config.ads[0],
            display_name: "vps-arm-sp-rust",
            shape: &sp_config.shape,
            shape_config: ShapeConfig {
                ocpus: sp_config.ocpus,
                memory_in_gbs: sp_config.ram_gb,
            },
            source_details: SourceDetails {
                source_type: "image",
                image_id: &sp_config.image_id,
                boot_volume_size_in_gbs: sp_config.boot_volume_gb,
            },
            create_vnic_details: VnicDetails {
                subnet_id: &sp_config.subnet_id,
                assign_public_ip: true,
                display_name: "vnic-sp-rust",
            },
            metadata,
        };

        let body_bytes = serde_json::to_vec(&payload).unwrap();
        let host = format!("iaas.{}.oraclecloud.com", sp_config.region);
        let path_query = "/20160918/instances";
        let date_header = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let (auth_header, sha_b64, cl) = signer
            .build_auth_header("POST", path_query, &host, &date_header, Some(&body_bytes))
            .unwrap();

        let url = format!("https://{}{}", host, path_query);

        let mut req = http_client
            .post(&url)
            .header("date", date_header)
            .header("host", &host)
            .header("authorization", auth_header)
            .header("content-type", "application/json")
            .header("content-length", cl.unwrap().to_string());

        if let Some(sha) = sha_b64 {
            req = req.header("x-content-sha256", sha);
        }

        let resp_result = req.body(body_bytes).send().await;

        match resp_result {
            Ok(resp) => {
                let status = resp.status();
                let body_text = resp.text().await.unwrap_or_default();

                if status.is_success() {
                    println!(
                        "\n\x1b[1;32m🎉 [SUCESSO {}] INSTÂNCIA ARM CRIADA COM SUCESSO!\x1b[0m",
                        now_time
                    );
                    println!("Resposta OCI: {}", body_text);
                    let msg = format!(
                        "Instância ARM criada com sucesso via Rust! Status: {}",
                        status
                    );
                    update_status_json(&sp_config.name, attempt, &now_time, &msg, None);
                    break;
                } else if status.as_u16() == 429 {
                    let cooldown = rng.gen_range(180..240);
                    let msg = format!("Rate-limit 429 detectado. Cooldown de {}s...", cooldown);
                    println!("[{}] ⚠️ {}", now_time, msg);
                    update_status_json(&sp_config.name, attempt, &now_time, &msg, None);
                    tokio::time::sleep(Duration::from_secs(cooldown)).await;
                } else if body_text.contains("Out of host capacity") {
                    let wait_sec = rng.gen_range(65..110);
                    let msg = format!(
                        "Tentativa #{}: Capacidade cheia. Próxima checagem em {}s...",
                        attempt, wait_sec
                    );
                    println!("[{}] {}", now_time, msg);
                    update_status_json(&sp_config.name, attempt, &now_time, &msg, None);
                    attempt += 1;
                    tokio::time::sleep(Duration::from_secs(wait_sec)).await;
                } else {
                    println!("[{}] Retorno OCI ({}): {}", now_time, status, body_text);
                    let wait_sec = rng.gen_range(65..110);
                    attempt += 1;
                    tokio::time::sleep(Duration::from_secs(wait_sec)).await;
                }
            }
            Err(e) => {
                println!("[{}] Erro de conexão HTTP: {}", now_time, e);
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        }
    }
}
