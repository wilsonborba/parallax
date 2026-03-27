# 09 - E2E Multi-Stream Test Playbook

## Objetivo

Validar fluxo real de ate 3 monitores no host + client Quest, incluindo:

- add/remove monitor sob demanda
- start/stop por monitor
- bloqueio de remocao de display em uso
- metadados de stream (`width,height,fps,bitrate,running`)

## Pre-requisitos

- Sessao Linux com X11 ativa (`echo $DISPLAY` e `xrandr --query` funcionando)
- Host e client no mesmo segmento de rede
- APK atualizado no Quest

## 1) Build e diagnostico host

```bash
cd /home/wilsonborba/Documents/Others/Asodya/parallax
cargo build --manifest-path host/Cargo.toml --bin prlx-hostd
cargo run --manifest-path host/Cargo.toml --bin prlx-hostd -- --virtual-backend-status
```

Esperado:

- `xrandr_available=true`
- `xrandr_setmonitor_supported=true`

## 2) Limpeza e criacao de 3 displays virtuais

```bash
cargo run --manifest-path host/Cargo.toml --bin prlx-hostd -- --disable-virtual-display prlx-v1 || true
cargo run --manifest-path host/Cargo.toml --bin prlx-hostd -- --disable-virtual-display prlx-v2 || true
cargo run --manifest-path host/Cargo.toml --bin prlx-hostd -- --disable-virtual-display prlx-v3 || true

cargo run --manifest-path host/Cargo.toml --bin prlx-hostd -- --enable-virtual-display prlx-v1 --virtual-width 1920 --virtual-height 1080 --virtual-x 1920 --virtual-y 0
cargo run --manifest-path host/Cargo.toml --bin prlx-hostd -- --enable-virtual-display prlx-v2 --virtual-width 1920 --virtual-height 1080 --virtual-x 3840 --virtual-y 0
cargo run --manifest-path host/Cargo.toml --bin prlx-hostd -- --enable-virtual-display prlx-v3 --virtual-width 1920 --virtual-height 1080 --virtual-x 5760 --virtual-y 0

xrandr --listmonitors
cargo run --manifest-path host/Cargo.toml --bin prlx-hostd -- --list-virtual-displays
```

## 3) Subir host daemon

```bash
cargo run --manifest-path host/Cargo.toml --bin prlx-hostd -- --display :0.0
```

Anotar no log:

- `Control listener bound on 0.0.0.0:<porta>`
- `Status socket bound on ".../prlx.sock"`

## 4) Parear no Quest

No Quest:

1. Abrir app
2. Escanear QR do host
3. Inserir PIN
4. Clicar `Start stream`

Esperado:

- status `Streaming`
- client mostra painel de monitores com `prlx-v1/v2/v3` apos `Refresh`

## 5) Teste funcional por monitor (client)

Para cada painel `prlx-v1`, `prlx-v2`, `prlx-v3`:

1. Clicar `Start`
2. Aguardar 5-10s
3. Clicar `Stop`

Esperado:

- alternancia `stopped -> running -> stopped`
- outros paineis permanecem estaveis

## 6) Teste de remocao protegida

1. Iniciar um painel (`Start`)
2. Tentar remover o mesmo painel (`-`)

Esperado:

- erro: display em uso por stream ativo
- display nao removido

3. Parar o painel (`Stop`) e remover novamente (`-`)

Esperado:

- remocao concluida

## 7) Validar metadados de stream

No client, usar `Refresh` apos alguns segundos de stream ativo.

Esperado em `ListStreams` no host (payload interno):

- colunas por linha: `stream_id,display,bind_addr,target_addr,prefer_vaapi,running,width,height,fps,bitrate_kbps`
- `width/height > 0`
- `fps > 0` durante stream ativo
- `bitrate_kbps > 0` durante stream ativo

Opcional (diagnostico via status socket do host):

```bash
# caminho padrao:
SOCK="$HOME/.local/share/prlx/prlx.sock"

# listar streams e metadados
printf 'streams\n' | socat - UNIX-CONNECT:"$SOCK"

# iniciar/parar stream especifico
printf 'start 2\n' | socat - UNIX-CONNECT:"$SOCK"
printf 'stop 2\n'  | socat - UNIX-CONNECT:"$SOCK"
```

## 8) Critérios de aceite

- add/remove sob demanda funciona sem reiniciar sessao
- start/stop por painel funciona
- remover display ativo e bloqueado corretamente
- metricas de `ListStreams` atualizam durante stream

## Observacoes

- O comando do status socket (`status/start/stop`) controla apenas stream 1 e nao substitui os testes de controle por `stream_id` via client.
- Se houver Wayland, executar em sessao X11 para os testes de monitor virtual.
