import subprocess
import time
import os

rust_exe = r"C:\Users\kbell\Downloads\bot-template-rust\target\release\bot-template-rust.exe"
go_exe = r"C:\Users\kbell\Downloads\bot-template-go\bot-app-windows.exe"

print(f"Tamanho Executavel Go:   {os.path.getsize(go_exe) / (1024*1024):.2f} MB")
print(f"Tamanho Executavel Rust: {os.path.getsize(rust_exe) / (1024*1024):.2f} MB")

env_rust = os.environ.copy()
env_rust["PORT"] = "8085"

p_rust = subprocess.Popen([rust_exe], cwd=r"C:\Users\kbell\Downloads\bot-template-rust", env=env_rust)
p_go = subprocess.Popen([go_exe], cwd=r"C:\Users\kbell\Downloads\bot-template-go")

time.sleep(3)

try:
    cmd_rust = ["tasklist", "/FI", "IMAGENAME eq bot-template-rust.exe", "/FO", "CSV"]
    out_r = subprocess.check_output(cmd_rust, text=True, errors="replace")
    for line in out_r.strip().splitlines()[1:]:
        print("Rust Process:", line)

    cmd_go = ["tasklist", "/FI", "IMAGENAME eq bot-app-windows.exe", "/FO", "CSV"]
    out_g = subprocess.check_output(cmd_go, text=True, errors="replace")
    for line in out_g.strip().splitlines()[1:]:
        print("Go Process:  ", line)
finally:
    p_rust.terminate()
    p_go.terminate()
    p_rust.wait()
    p_go.wait()
