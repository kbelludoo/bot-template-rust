import oci
import time
import sys
import json
import random
from datetime import datetime

sys.stdout.reconfigure(encoding='utf-8')

config = oci.config.from_file('C:/Users/kbell/.oci/config')
compute_client = oci.core.ComputeClient(config)
network_client = oci.core.VirtualNetworkClient(config)

tenancy_id = config['tenancy']
subnet_id = 'ocid1.subnet.oc1.iad.aaaaaaaangw73b6y3bi4nk5526ablwgnhezez7yyzesuufegdkwv3cob3y6q'
image_id = 'ocid1.image.oc1.iad.aaaaaaaa6o52ajhkewa2syfadrxel5b7tfwi5qdm45gl4ykwbqut25tvdxrq' # Ubuntu 24.04 aarch64

ssh_key_1 = 'ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDFuMXGbJm6U99H8SEhdItQ1n/2DbtVCMyZsXIR6v+xPMYfqS45wvQaHz9w9SAcaC88dHf02LJe9omsu3oqViZKurt51GZVsJYBrsLNZttGyu8qrEIPPKFoajW2EOl+9iwUfxh0O4s8MypzeGvFhhlIymiG2EFaFXSQBAD4lvVLtAMYYPDjlQAFSDv8qeXw/A1WWIFpBqaGSXdJXk296MvYpjs17L9uhn0uc1r4KPazNzgJIw+p4hb9rkO7t706kOTBNfXoznfKqGsMbucId9hqDJJHcO265Rp7HX3HO/LA5Kh23+HMVZD8EzxQmznl2WEwyqC+r8cggu1Bm+tNF63D ssh-key-2026-10-06'
ssh_key_2 = 'ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCmG4gVhzRv1TQT6xOFXChva3jcNeqk4PHquOYfNgVAP77gSfh5yujJwCW/yHYLpo4RqIjnn7qFA93qlBvlmrc6R6TvekmPDuPWLoKO+Bo7n0kMKovB9oPHNXjMiAPfM2mOr1PWXXTK4mvxh4SQQbY/IQeh0dDuOi/xingmzLkE0vrnMowZCKJbBM2W7jlxHqSvRIsg+6pkJ2kip1qeFEM9cbWKkNFJMvc4arPIWQwDxOAa7OIAZjtOOlkaHUJI9himaVEPL1Q5Ol3DYNIFmeEfC+z02twdKJm55qABUFTrU5vNkoYGVo+KzYlpEuINOtvHw4N8ZwSl3g0WQHf0deFR ssh-key-2026-10-12'

combined_keys = f'{ssh_key_1}\n{ssh_key_2}'

ads = ['vkRB:US-ASHBURN-AD-1', 'vkRB:US-ASHBURN-AD-2', 'vkRB:US-ASHBURN-AD-3']

print("=" * 65, flush=True)
print("🛡️  SNIPER ARM COM PROTEÇÃO ANTI-RATE-LIMIT (STEALTH MODE)", flush=True)
print("Config: 2 OCPUs, 12 GB RAM, Intervalo seguro: 90s-140s + Jitter", flush=True)
print("=" * 65, flush=True)

attempt = 1
created_instance = None

while not created_instance:
    now = datetime.now().strftime("%H:%M:%S")
    hit_rate_limit = False

    # Embaralhar a ordem dos ADs para evitar padrão fixo de varredura
    shuffled_ads = list(ads)
    random.shuffle(shuffled_ads)

    for ad in shuffled_ads:
        short_ad = ad.split("-")[-1]
        launch_details = oci.core.models.LaunchInstanceDetails(
            compartment_id=tenancy_id,
            availability_domain=ad,
            display_name='vps-arm-master',
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
                display_name='primary-vnic'
            ),
            metadata={
                'ssh_authorized_keys': combined_keys
            }
        )
        try:
            res = compute_client.launch_instance(launch_details)
            created_instance = res.data
            print(f"\n🎉 [SUCESSO {now}] Instância A1 criada em {short_ad}!", flush=True)
            print(f"ID da Instância: {created_instance.id}", flush=True)
            break
        except oci.exceptions.ServiceError as e:
            if "TooManyRequests" in str(e.code) or "Too many requests" in str(e.message):
                print(f"[{now}] ⚠️ Rate-limit detectado no AD-{short_ad}. Pausando de forma defensiva...", flush=True)
                hit_rate_limit = True
                break
            elif "Out of host capacity" in str(e.message):
                pass
            else:
                print(f"[{now}] AD-{short_ad} Erro: {e.code} - {e.message}", flush=True)

        # Pausa suave entre um AD e outro (4 a 8 segundos) para não bater tudo no mesmo segundo
        time.sleep(random.uniform(4.0, 8.0))

    if created_instance:
        break

    if hit_rate_limit:
        # Se a Oracle avisar TooManyRequests, recua por 3 a 4 minutos para esfriar completamente
        cooldown = random.randint(180, 240)
        print(f"[{now}] Cooldown de segurança: aguardando {cooldown}s para limpar o rate-limit...", flush=True)
        time.sleep(cooldown)
    else:
        # Intervalo seguro padrão com jitter humano (90s a 140s)
        wait_sec = random.randint(90, 140)
        print(f"[{now}] Tentativa #{attempt}: Capacidade cheia. Próxima checagem em {wait_sec}s...", flush=True)
        attempt += 1
        time.sleep(wait_sec)

print("\nAguardando a nova VPS ficar em estado RUNNING...", flush=True)
inst_id = created_instance.id
while True:
    st = compute_client.get_instance(inst_id).data
    print(f"Estado atual: {st.lifecycle_state}", flush=True)
    if st.lifecycle_state == 'RUNNING':
        break
    time.sleep(10)

# Obter VNIC e IP Publico
vnic_attachments = compute_client.list_vnic_attachments(tenancy_id, instance_id=inst_id).data
primary_vnic = None
for va in vnic_attachments:
    v = network_client.get_vnic(va.vnic_id).data
    if v.is_primary:
        primary_vnic = v
        break

info = {
    "instance_id": inst_id,
    "display_name": st.display_name,
    "availability_domain": st.availability_domain,
    "shape": st.shape,
    "ocpus": 2.0,
    "ram_gb": 12.0,
    "primary_public_ip": primary_vnic.public_ip if primary_vnic else None,
    "primary_private_ip": primary_vnic.private_ip if primary_vnic else None,
    "created_at": datetime.now().isoformat()
}

with open("new_vps_info.json", "w", encoding="utf-8") as f:
    json.dump(info, f, indent=2)

print("\n" + "=" * 65, flush=True)
print(f"✅ VPS ARM MASTER ATIVA! IP: {info['primary_public_ip']}", flush=True)
print("=" * 65, flush=True)
