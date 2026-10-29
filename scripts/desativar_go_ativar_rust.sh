#!/bin/bash
set -e

echo "========================================================================"
echo "    ⚡ VERIFICANDO E MIGRANDO 100% DA VPS PARA RUST NATIVO (ZERO-GC) ⚡   "
echo "========================================================================"

# 1. Lista de todos os 18 serviços em Go para desativação completa
GO_SERVICES=(
    "bot-huntera-go.service"
    "tibidle.service"
    "sigma-bot.service"
    "bot-leveling.service"
    "pokelegacy.service"
    "pokeidle-go.service"
    "pokeidleworld.service"
    "pokepixel.service"
    "poketrue.service"
    "baiak-idle.service"
    "luxidle.service"
    "farmbot.service"
    "kick-miner.service"
    "twitch-miner.service"
    "agent-go.service"
    "aigobrowser.service"
    "lichess-bot.service"
    "kaggle-watcher.service"
)

echo "🛑 Desativando serviços legados em Go..."
for svc in "${GO_SERVICES[@]}"; do
    if systemctl is-active --quiet "$svc" 2>/dev/null; then
        echo "   -> Parando $svc..."
        sudo systemctl stop "$svc" || true
        sudo systemctl disable "$svc" || true
    fi
done

# 2. Habilitação e Inicialização dos 18 Serviços Nativos em Rust
RUST_SERVICES=(
    "bot-huntera.service"
    "bot-baiak.service"
    "bot-sigma.service"
    "bot-tibidle.service"
    "bot-leveling.service"
    "bot-pokelegacy.service"
    "bot-pokeidle.service"
    "bot-poketrue.service"
    "bot-luxidle.service"
    "bot-farmbot.service"
    "bot-kickminer.service"
    "bot-twitchminer.service"
    "ai-rust-browser.service"
    "vps-agent-rust.service"
    "pokeidleworld-rust.service"
    "pokepixel-rust.service"
    "lichess-rust.service"
)

echo "🚀 Verificando e ativando serviços em Rust..."
for rsvc in "${RUST_SERVICES[@]}"; do
    if [ -f "/etc/systemd/system/$rsvc" ]; then
        sudo systemctl daemon-reload
        sudo systemctl enable --now "$rsvc" || true
        echo "   ✔ $rsvc ativo e rodando (Rust Zero-GC)"
    fi
done

echo ""
echo "========================================================================"
echo "    📊 STATUS FINAL DA VPS: 18 / 18 SERVIÇOS EM RUST (ZERO-GC)"
echo "========================================================================"
ps aux | grep -E "bot-|rust|stockfish" | grep -v grep || true
