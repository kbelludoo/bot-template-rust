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
image_id = 'ocid1.image.oc1.sa-saopaulo-1.aaaaaaaaqqxqqfkhlzjcneto533jgu7ey6nzlzlpt5kuh6cqtbldhy2bu2hq' # Canonical-Ubuntu-24.04-aarch64

ssh_key_sp1 = 'ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDSx8Loag4kbo5jj2p4+REZzO6TanBxPDTKyCavdIQKKcIfoyVJgS7cnNAIfEZdV5S5jbH7xmOXGz5B3EXJngScCAHawyve2MPyR6C1z/UNduhsR4PvHlyWldvVLvT1lNXjGIwVGh21maVHPdFK2YHA1up7Zx3ttfyqSzxQX19/gyx805DLTm5EA7MF1ja6p8J4N6JACqnfTMVTkDwKUNNSxOXdCuYh3IZkDgJ9QdGhHm8D7OwmvyZdzgrZ2F5q8upP1F1b3F80gh9XMJaLJFK5xUFzWJykudFiqz5sRUUVU/LlUjsEKQjPFtXtlsbL1XKv1tDbhHO8wouTI5EQxdT5 ssh-key-2026-10-09'
ssh_key_sp2 = 'ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDFhNsjuvnJZ6/MMhXDZLL8ee1C8D2+Wu3GD0R/ugb/wLzTAVlFpf0jMh4HOtUCWb6q241H8LOODyEN44KQDf7bQaQukT4j9yHj1Y87LRaQle931wl4ugu7hO5GQ19nnSSQDV7kh1oHYLYp1abnfRJvagIomewOSuvFQKyQA92IxKxu90e9KMlf+OcayY4bgu4gJEWNNA9nbN6zTRV/qEySx+FbB61jw1rPCg2KaH1rsn12smWJjwkFMTxMomHicLUOQ1PFvIRDF0lhTeUCSthFZOmTve+f5snBtPyb8puyCRMsZWGPtBOM/Mp0yXbqTdO9PVpZCPtj/UdkgoJ+Z2fR ssh-key-2026-10-10'

combined_keys = f'{ssh_key_sp1}\n{ssh_key_sp2}'

ad_name = 'vzaS:SA-SAOPAULO-1-AD-1'

print("=" * 65, flush=True)
print("🛡️  SNIPER ARM SÃO PAULO (STEALTH MODE)", flush=True)
print("Config: 2 OCPUs, 12 GB RAM, 100 GB Boot Volume, Ubuntu 24.04 ARM", flush=True)
print("Intervalo seguro: 90s-140s + Jitter", flush=True)
print("=" * 65, flush=True)

attempt = 1
created_instance = None

while not created_instance:
    now = datetime.now().strftime("%H:%M:%S")
    hit_rate_limit = False

    launch_details = oci.core.models.LaunchInstanceDetails(
        compartment_id=tenancy_id,
        availability_domain=ad_name,
        display_name='vps-arm-saopaulo-master',
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
        print(f"\n🎉 [SUCESSO {now}] Instância A1 criada em São Paulo!", flush=True)
        print(f"ID da Instância: {created_instance.id}", flush=True)
        break
    except oci.exceptions.ServiceError as e:
        if "TooManyRequests" in str(e.code) or "Too many requests" in str(e.message):
            print(f"[{now}] ⚠️ Rate-limit detectado. Pausando de forma defensiva...", flush=True)
            hit_rate_limit = True
        elif "Out of host capacity" in str(e.message):
            pass
        else:
            print(f"[{now}] Erro OCI: {e.code} - {e.message}", flush=True)

    if created_instance:
        break

    if hit_rate_limit:
        cooldown = random.randint(180, 240)
        print(f"[{now}] Cooldown de segurança: aguardando {cooldown}s para limpar o rate-limit...", flush=True)
        time.sleep(cooldown)
    else:
        wait_sec = random.randint(90, 140)
        print(f"[{now}] Tentativa #{attempt}: Capacidade cheia. Próxima checagem em {wait_sec}s...", flush=True)
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
for v in vnics:
    vnic = network_client.get_vnic(v.vnic_id).data
    print(f"🎯 IP PÚBLICO DA NOVA VPS SP: {vnic.public_ip}", flush=True)
    print(f"🔒 IP PRIVADO: {vnic.private_ip}", flush=True)

print("\n🚀 VPS ARM SÃO PAULO PRONTA COM SUCESSO!", flush=True)
