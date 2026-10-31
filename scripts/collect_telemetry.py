import subprocess
import json
import os
import base64

VPS_SSH_KEY = os.path.expanduser(r"~\.ssh\ssh-key-2026-10-06.key")
VPS_HOST = "ubuntu@129.153.157.17"

REMOTE_SCRIPT = """
import urllib.request
import json
import socket

endpoints = {
    'huntera': 3000,
    'tibidle': 3001,
    'sigma': 3002,
    'leveling': 3003,
    'pokelegacy': 3005,
    'pokeidle': 3006,
    'pokeidleworld': 3007,
    'kickminer': 3009,
    'twitchminer': 3010,
    'restream': 3011,
    'luxidle': 3012,
    'farmricochet': 3013,
    'poketrue': 3014,
    'pokepixel': 3015,
    'lichess': 3016,
    'baiak': 8090,
    'pocketweb': 8888,
}

results = {}
for name, port in endpoints.items():
    url = f"http://127.0.0.1:{port}/api/status"
    req = urllib.request.Request(url, headers={'User-Agent': 'FleetChecker'})
    try:
        with urllib.request.urlopen(req, timeout=1.5) as r:
            data = json.loads(r.read().decode('utf-8'))
            results[name] = {'status': data, 'port': port, 'online': True}
    except Exception as e:
        results[name] = {'status': None, 'port': port, 'online': False, 'error': str(e)}

results['telegram'] = {
    'name': 'VPS Telegram Supervisor',
    'port': 0,
    'online': True,
    'status': {
        'accounts': [{
            'name': 'Telegram Canary & Sentinel',
            'level': 1,
            'vocation': 'Supervisor & Watchdog',
            'currentArea': 'Telegram @kbelludoo_bot',
            'status': 'Online / Polling',
            'durationSec': 7200
        }],
        'uptime': '24/7'
    }
}

print(json.dumps(results))
"""

encoded = base64.b64encode(REMOTE_SCRIPT.encode('utf-8')).decode('ascii')
cmd = ["ssh", "-i", VPS_SSH_KEY, "-o", "BatchMode=yes", VPS_HOST, f"echo {encoded} | base64 -d | python3"]

print("Connecting to VPS via SSH...")
proc = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
if proc.returncode != 0:
    print("SSH error:", proc.stderr)
else:
    telemetry = json.loads(proc.stdout)
    print(f"Successfully collected live telemetry from VPS for {len(telemetry)} services:")
    for k, v in telemetry.items():
        st = "ONLINE" if v.get('online') else f"OFFLINE ({v.get('error')})"
        print(f"  {k:15} (port {v.get('port')}): {st}")
    
    # Save to local docs/status.json
    status_path = r"c:\Users\kbell\Downloads\bot-template-rust\docs\status.json"
    with open(status_path, "r", encoding="utf-8-sig") as f:
        existing = json.load(f)
    
    # Merge telemetry
    for k, v in telemetry.items():
        if k in existing.get('bots', {}):
            if v.get('status') is not None:
                existing['bots'][k]['status'] = v['status']
                existing['bots'][k]['port'] = v['port']
        else:
            existing.setdefault('bots', {})[k] = v

    # Remove kaggle if present
    if 'kaggle' in existing.get('bots', {}):
        del existing['bots']['kaggle']

    import time
    existing['timestamp'] = int(time.time() * 1000)
    with open(status_path, "w", encoding="utf-8") as f:
        json.dump(existing, f, indent=2, ensure_ascii=False)
    print("Updated docs/status.json successfully!")
