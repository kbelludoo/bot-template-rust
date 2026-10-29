# ⚡ Game Bot Starter Kit & Command Center (Rust Native Engine)

Template padronizado de altíssima performance desenvolvido em **Rust nativo (Zero-GC)** para bots autônomos de jogos de navegador / idle com consumo de memória ultra-reduzido (~1 a 3 MB de RAM).

Transposto fielmente a partir do ecossistema de referência `bot-template-go`.

---

## 🌟 Recursos Principais

- ⚡ **Zero Garbage Collection:** Alocações determinísticas na Stack, sem pausas de GC, ideal para rodar dezenas de instâncias em VPS pequenas.
- 🛡️ **Anti-Ban Circuit Breaker (`src/engine/circuit_breaker.rs`):** Disjuntor automático que detecta erros 429, 403 e recusas heurísticas (`experiência rápida demais`, `save malformado`). Desarma o combate e aguarda cooldown com backoff exponencial.
- 💤 **Auditoria de Stamina & Fadiga (`src/engine/stamina.rs`):** Controlador de histerese anti-thrashing com suporte a fadiga percentual (0% a 100%) e stamina decrescente (42h a 0h).
- 🧩 **Preservação de Schema Bruto (`src/engine/overlay.rs`):** `StateOverlay` preserva campos desconhecidos do servidor prevenindo saves malformados.
- 🌐 **Networking Resiliente (`src/network`):**
  - Headers furtivos modernos (`sec-ch-ua`, User-Agent).
  - Assinador HMAC-SHA256 (`signer.rs`) com idempotência UUIDv4 e particionamento hash DJB2.
  - Cliente WebSocket resiliente (`socket.rs`).
- 📦 **Suíte Multi-Protocolo (`src/protocol`):**
  - `binary.rs`: Varint, Zigzag e decodificador Protobuf com zero-alocação.
  - `socketio.rs`: Parser de pacotes Engine.IO / Socket.IO v4.
- 🧩 **Subsistemas Autônomos (`src/subsystem`):**
  - `RouteSolver`: Cálculo da melhor rota por nível.
  - `InventoryManager`: Auto-sell de lixo e controle de mochila (>80%).
  - `DailyManager`: Resgate de streaks diários na virada UTC.
  - `PotionManager`: Otimizador de poções HP/Mana.
- 🌐 **Web Dashboard Embutido (Axum):**
  - Servidor web assíncrono embutido na porta `8080` com rota `/api/status`.
  - Console Ticker em tempo real no terminal.

---

## 🚀 Como Executar

```bash
cd C:\Users\kbell\Downloads\bot-template-rust

# 1. Rodar os testes unitários
cargo test

# 2. Executar em modo desenvolvimento
cargo run

# 3. Compilar em modo release ultra-otimizado (binário stripped, LTO)
cargo build --release
```

Os arquivos de configuração seguem o mesmo formato:
- `accounts.example.json` (modelo)
- `accounts.json` (credenciais das suas contas)
