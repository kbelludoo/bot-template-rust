import oci
import time
import sys
import json
import random
from datetime import datetime

sys.stdout.reconfigure(encoding='utf-8')

config = oci.config.from_file('C:/Users/kbell/.oci/config', profile_name='SAOPAULO')
compute_client = oci.core.ComputeClient(config)
network_client = oci.core.VirtualNetworkClient(config)

tenancy_id = config['tenancy']
subnet_id = 'ocid1.subnet.oc1.sa-saopaulo-1.aaaaaaaamhykqtiezgqujpjb3qwpt7sx6be7kduziztuf5q7g5jdjgk3vxga'
# Canonical-Ubuntu-26.04-Minimal-aarch64
image_id = 'ocid1.image.oc1.sa-saopaulo-1.aaaaaaaamgy43642gy2ta7pdh6etf6ye5kspe5u5bvj4hxvhrgjqwi3lohkq'

ssh_key_sp1 = 'ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDSx8Loag4kbo5jj2p4+REZzO6TanBxPDTKyCavdIQKKcIfoyVJgS7cnNAIfEZdV5S5jbH7xmOXGz5B3EXJngScCAHawyve2MPyR6C1z/UNduhsR4PvHlyWldvVLvT1lNXjGIwVGh21maVHPdFK2YHA1up7Zx3ttfyqSzxQX19/gyx805DLTm5EA7MF1ja6p8J4N6JACqnfTMVTkDwKUNNSxOXdCuYh3IZkDgJ9QdGhHm8D7OwmvyZdzgrZ2F5q8upP1F1b3F80gh9XMJaLJFK5xUFzWJykudFiqz5sRUUVU/LlUjsEKQjPFtXtlsbL1XKv1tDbhHO8wouTI5EQxdT5 ssh-key-2026-10-09'
ssh_key_sp2 = 'ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDFhNsjuvnJZ6/MMhXDZLL8ee1C8D2+Wu3GD0R/ugb/wLzTAVlFpf0jMh4HOtUCWb6q241H8LOODyEN44KQDf7bQaQukT4j9yHj1Y87LRaQle931wl4ugu7hO5GQ19nnSSQDV7kh1oHYLYp1abnfRJvagIomewOSuvFQKyQA92IxKxu90e9KMlf+OcayY4bgu4gJEWNNA9nbN6zTRV/qEySx+FbB61jw1rPCg2KaH1rsn12smWJjwkFMTxMomHicLUOQ1PFvIRDF0lhTeUCSthFZOmTve+f5snBtPyb8puyCRMsZWGPtBOM/Mp0yXbqTdO9PVpZCPtj/UdkgoJ+Z2fR ssh-key-2026-10-10'

combined_keys = f'{ssh_key_sp1}\n{ssh_key_sp2}'
ad_name = 'vzaS:SA-SAOPAULO-1-AD-1'

def update_status_json(attempt, last_time, msg, created=None):
    try:
        status_path = 'docs/sniper-status.json'
        with open(status_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
        if 'saopaulo' in data['regions']:
            sp = data['regions']['saopaulo']
            sp['totalAttempts'] = attempt
            sp['lastAttemptTime'] = last_time
            sp['lastResult'] = msg
            if created:
                sp['status'] = 'active'
                sp['createdInstance'] = created
        data['logs'].insert(0, {
            'time': last_time,
            'region': 'saopaulo',
            'level': 'INFO' if not created else 'SUCCESS',
            'message': msg
        })
        data['logs'] = data['logs'][:25]
        with open(status_path, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
    except Exception:
        pass

print("=" * 65, flush=True)
print("🛡️  SNIPER ARM SÃO PAULO — UBUNTU 26.04 MINIMAL (2 OCPU / 12 GB RAM)", flush=True)
print("Internet 100% Liberada | 100 GB Boot NVMe | Modo Stealth Ativo", flush=True)
print("=" * 65, flush=True)

attempt = 1
created_instance = None

while not created_instance:
    now = datetime.now().strftime("%H:%M:%S")
    hit_rate_limit = False

    launch_details = oci.core.models.LaunchInstanceDetails(
        compartment_id=tenancy_id,
        availability_domain=ad_name,
        display_name='vps-arm-sp-master',
        shape='VM.Standard.A1.Flex',
        shape_config=oci.core.models.LaunchInstanceShapeConfigDetails(
            ocpus=2.0,
            memory_in_gbs=12.0
        ),
        source_details=oci.core.models.InstanceSourceViaImageDetails(
            image_id=image_id,
            boot_volume_size_in_gbs=100
        ),
        create_vnic_details=oci.core.models.CreateVnicDetails(
            subnet_id=subnet_id,
            assign_public_ip=True,
            display_name='primary-vnic-sp'
        ),
        metadata={
            'ssh_authorized_keys': combined_keys
        }
    )

    try:
        res = compute_client.launch_instance(launch_details)
        created_instance = res.data
        print(f"\n🎉 [SUCESSO {now}] INSTÂNCIA ARM CRIADA EM SÃO PAULO!", flush=True)
        print(f"ID da Instância: {created_instance.id}", flush=True)
        break
    except oci.exceptions.ServiceError as e:
        if "TooManyRequests" in str(e.code) or "Too many requests" in str(e.message):
            print(f"[{now}] ⚠️ Rate-limit detectado. Pausando...", flush=True)
            hit_rate_limit = True
        elif "Out of host capacity" in str(e.message):
            pass
        else:
            print(f"[{now}] Erro OCI: {e.code} - {e.message}", flush=True)

    if hit_rate_limit:
        cooldown = random.randint(180, 240)
        print(f"[{now}] Cooldown de segurança: {cooldown}s...", flush=True)
        time.sleep(cooldown)
    else:
        wait_sec = random.randint(65, 110)
        msg = f"Tentativa #{attempt}: Capacidade cheia. Próxima checagem em {wait_sec}s..."
        print(f"[{now}] {msg}", flush=True)
        update_status_json(attempt, now, msg)
        attempt += 1
        time.sleep(wait_sec)

print("\nAguardando a nova VPS de São Paulo ficar em estado RUNNING...", flush=True)
inst_id = created_instance.id

while True:
    inst = compute_client.get_instance(inst_id).data
    print(f"Estado atual: {inst.lifecycle_state}", flush=True)
    if inst.lifecycle_state == 'RUNNING':
        break
    time.sleep(10)

print("\nBuscando IP Público atribuído...", flush=True)
vnics = compute_client.list_vnic_attachments(compartment_id=tenancy_id, instance_id=inst_id).data
public_ip = ""
private_ip = ""
for v in vnics:
    vnic = network_client.get_vnic(v.vnic_id).data
    public_ip = vnic.public_ip
    private_ip = vnic.private_ip
    print(f"🎯 IP PÚBLICO: {public_ip}", flush=True)
    print(f"🔒 IP PRIVADO: {private_ip}", flush=True)

update_status_json(attempt, datetime.now().strftime("%H:%M:%S"), f"VPS ARM criada com sucesso! IP: {public_ip}", {
    'id': inst_id,
    'public_ip': public_ip,
    'private_ip': private_ip,
    'state': 'RUNNING'
})

print("\n🚀 VPS ARM SÃO PAULO 100% ONLINE E PRONTA!", flush=True)
