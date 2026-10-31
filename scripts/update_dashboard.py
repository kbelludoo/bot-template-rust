import os

index_path = r"c:\Users\kbell\Downloads\bot-template-rust\docs\index.html"
with open(index_path, "rb") as f:
    raw = f.read()

text = raw.decode("latin-1")

# 1. Remove card-kaggle
k_start = text.find('<div class="bot-card" id="card-kaggle"')
k_end = text.find('<div class="bot-card" id="card-telegram"')

if k_start != -1 and k_end != -1:
    print(f"Removing card-kaggle from index.html ({k_start} to {k_end})...")
    text = text[:k_start] + text[k_end:]
else:
    print("card-kaggle not found or already removed.")

# 2. Update getUnifiedAccounts fallback
old_block_marker = "if (b.status.uptime || b.status.ramSysMB || b.status.goroutines) {"
new_block = """      if (b.status.bot_user) {
        return [{
          name: '@' + b.status.bot_user,
          level: (b.status.wins || 0) + ' V / ' + (b.status.losses || 0) + ' D',
          vocation: 'Stockfish 19 NNUE',
          currentArea: b.status.idle_learner && b.status.idle_learner.status ? b.status.idle_learner.status : 'Berserk / Lichess TV',
          gold: b.status.moves_played || 0,
          kills: b.status.games_played || 0,
          status: b.status.active_games > 0 ? 'Em Partida' : 'Ocioso / Aprendendo',
          durationSec: b.status.uptime_sec || 0,
          isChess: true
        }];
      }

      if (b.status.targets || (b.name && b.name.indexOf('Restream') !== -1)) {
        return [{
          name: 'Relay Twitch / Kick / YT',
          level: b.status.active_accounts || 3,
          vocation: 'Rust Zero-GC Relay',
          currentArea: 'Transmissão Multicast 720p 10fps',
          gold: b.status.frames_emitted || 0,
          kills: 3,
          status: 'Transmitindo Ao Vivo',
          durationSec: b.status.uptime_seconds || 0,
          isStream: true
        }];
      }

      if (b.status.online || b.status.uptime || b.status.uptime_seconds || b.status.uptime_sec || b.status.ramSysMB || b.status.goroutines) {
        return [{
          name: 'Sessão Conectada',
          level: 1,
          vocation: 'Operação Contínua',
          currentArea: 'Monitorando em Standby',
          status: 'Online',
          durationSec: b.status.uptime_seconds || b.status.uptime_sec || 0
        }];
      }"""

if old_block_marker in text:
    print("Updating getUnifiedAccounts in index.html...")
    # Find full block to replace
    start_pos = text.find(old_block_marker)
    # The block ends before `return [];`
    end_marker = "return [];"
    end_pos = text.find(end_marker, start_pos)
    text = text[:start_pos] + new_block + "\n\n      " + text[end_pos:]

with open(index_path, "wb") as f:
    f.write(text.encode("latin-1"))

print("index.html updated successfully!")
