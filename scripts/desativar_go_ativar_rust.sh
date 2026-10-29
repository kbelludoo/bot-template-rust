#!/bin/bash
set -e

echo "================================================================="
echo "   ⚡ DESATIVANDO SERVIÇOS LEGADOS GO E ATIVANDO RUST NA VPS ⚡   "
echo "================================================================="

# 1. Parar e desabilitar os serviços legados em Go
echo "🛑 Parando serviços em Go..."
sudo systemctl stop lichess-bot.service || true
sudo systemctl disable lichess-bot.service || true

sudo systemctl stop kick-miner.service || true
sudo systemctl disable kick-miner.service || true

sudo systemctl stop twitch-miner.service || true
sudo systemctl disable twitch-miner.service || true

sudo systemctl stop agent-go.service || true
sudo systemctl disable agent-go.service || true

# 2. Instalar e habilitar os novos serviços nativos em Rust
echo "🚀 Ativando serviços nativos em Rust (Zero-GC)..."
if [ -f "/home/ubuntu/bot-lichess-rust" ]; then
    sudo cp /home/ubuntu/bot-lichess-rust/lichess-rust.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable lichess-rust.service
    sudo systemctl restart lichess-rust.service
    echo "✔ Lichess Rust ativado com sucesso!"
fi

if [ -f "/home/ubuntu/bot-kickminer-rust" ]; then
    sudo systemctl enable kickminer-rust.service || true
    sudo systemctl restart kickminer-rust.service || true
    echo "✔ Kick Miner Rust ativado com sucesso!"
fi

if [ -f "/home/ubuntu/bot-twitchminer-rust" ]; then
    sudo systemctl enable twitchminer-rust.service || true
    sudo systemctl restart twitchminer-rust.service || true
    echo "✔ Twitch Miner Rust ativado com sucesso!"
fi

echo "================================================================="
echo "   🎉 100% DA VPS AGORA ESTÁ OPERANDO EM RUST NATIVO (ZERO-GC)!  "
echo "================================================================="
