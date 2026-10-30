# 🛡️ Oracle Cloud ARM Sniper Suite (Rust & Python Engine)

Suite de alta precisão e monitoramento contínuo para provisionamento automático de instâncias **Always Free ARM Ampere A1** (`VM.Standard.A1.Flex`) na Oracle Cloud Infrastructure (OCI).

Configurado para capturar instâncias de potência máxima:
- **2 OCPUs** (Ampere Altra aarch64)
- **12 GB de Memória RAM**
- **100 GB de Volume de Inicialização SSD NVMe**
- **IP Público Dedicado**
- **Ubuntu 24.04 LTS (Noble Numbat)**

---

## 🎯 Por que esta aplicação existe?

A Oracle disponibiliza gratuitamente até 4 OCPUs e 24 GB de RAM no programa *Always Free*. No entanto, os servidores físicos ARM nos datacenters (como Ashburn e São Paulo) frequentemente ficam com 100% de ocupação (`Out of host capacity`).

Esta aplicação opera como um **sniper inteligente** que:
1. Monitora o datacenter continuamente.
2. Utiliza **Jitter Dinâmico (90s a 140s)** com intervalos randômicos para evitar detecção de bots e bloqueio por *TooManyRequests* (HTTP 429).
3. Alterna a ordem de checagem dos domínios de disponibilidade (ADs) de forma aleatória.
4. No momento exato em que outro usuário desliga ou exclui uma máquina, a aplicação dispara a requisição e garante a criação da sua VPS.
5. Exporta telemetria em tempo real para um **Dashboard Web no GitHub Pages**.

---

## 📂 Estrutura do Projeto

```text
bot-template-rust/
├── oracle-sniper/
│   ├── Cargo.toml                  # Manifesto independente para Rust
│   ├── README.md                   # Este guia passo a passo completo
│   └── src/
│       └── main.rs                 # Orquestrador nativo em Rust & Telemetria
├── scripts/
│   ├── sniper_arm.py               # Motor Sniper OCI para Ashburn (US East)
│   └── sniper_arm_saopaulo.py      # Motor Sniper OCI para São Paulo (BR East)
└── docs/
    ├── sniper.html                 # Dashboard Web responsivo para acompanhar
    └── sniper-status.json          # Telemetria ao vivo consumida pela página
```

---

## 🚀 Passo a Passo: Configuração e Execução

### 1. Pré-requisitos
- **Rust & Cargo** instalados (edição 2021/2024).
- **Python 3.10+** com o pacote oficial da Oracle:
  ```bash
  pip install oci
  ```

### 2. Configuração das Credenciais OCI (`~/.oci/config`)
No Windows, o arquivo fica em `C:\Users\<SEU_USUARIO>\.oci\config`. No Linux/macOS, fica em `~/.oci/config`.

Exemplo com duas contas (EUA e Brasil):

```ini
[DEFAULT]
user=ocid1.user.oc1..aaaaaaaavw...
fingerprint=5a:29:59:9b:85:e2...
tenancy=ocid1.tenancy.oc1..aaaaaaaamwhan...
region=us-ashburn-1
key_file=C:\Users\kbell\.oci\ashburn_key.pem

[SAOPAULO]
user=ocid1.user.oc1..aaaaaaaavw35...
fingerprint=32:10:91:56:8a:0d...
tenancy=ocid1.tenancy.oc1..aaaaaaaagqnk...
region=sa-saopaulo-1
key_file=C:\Users\kbell\.oci\saopaulo_key.pem
```

> [!IMPORTANT]
> A chave privada `.pem` deve ter permissões restritas. No Linux: `chmod 600 ~/.oci/*.pem`.

---

### 3. Como Rodar com o Motor Rust

Você pode compilar e executar o orquestrador diretamente com o Cargo:

#### A. Ver o status atual e tentativas
```bash
cargo run --bin oracle-sniper status
```

#### B. Forçar atualização e sincronização da telemetria
```bash
cargo run --bin oracle-sniper update
```

#### C. Rodar o orquestrador em modo contínuo (Daemon)
```bash
cargo run --bin oracle-sniper daemon
```

---

### 4. Executando os Motores Snipers Diretamente

#### Para a Conta de Ashburn (US):
```bash
python -u scripts/sniper_arm.py
```

#### Para a Conta de São Paulo (BR):
```bash
python -u scripts/sniper_arm_saopaulo.py
```

---

### 5. Como Rodar 24/7 na sua VPS Linux (Systemd)

Para deixar o sniper rodando 24 horas por dia sem precisar deixar o computador ligado:

1. Crie o arquivo de serviço:
   ```bash
   sudo nano /etc/systemd/system/oracle-sniper.service
   ```

2. Cole a configuração:
   ```ini
   [Unit]
   Description=Oracle ARM Sniper Service
   After=network.target

   [Service]
   Type=simple
   User=ubuntu
   WorkingDirectory=/home/ubuntu/bot-template-rust
   ExecStart=/usr/bin/python3 -u scripts/sniper_arm_saopaulo.py
   Restart=always
   RestartSec=30

   [Install]
   WantedBy=multi-user.target
   ```

3. Ative e inicie o serviço:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable --now oracle-sniper
   sudo systemctl status oracle-sniper
   ```

---

## 🌐 Acompanhamento via Web (GitHub Pages)

Acompanhe o status e as tentativas em tempo real direto pelo navegador no painel interativo:

👉 **[https://kbelludoo.github.io/bot-template-rust/sniper.html](https://kbelludoo.github.io/bot-template-rust/sniper.html)**

O painel recarrega a telemetria automaticamente, exibindo:
- Status de cada região (Aguardando vaga / Ativa).
- Contador total de tentativas.
- Timestamp da última verificação.
- Log de eventos e resposta da API da Oracle.
- IP público alocado assim que a máquina for criada.
