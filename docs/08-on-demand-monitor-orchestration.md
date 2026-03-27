# 08 - On-Demand Monitor Orchestration (Client-Driven)

## Objetivo

Garantir fluxo sequencial e sob demanda para monitores no Quest:

- Usuario pede `+1 monitor` no client.
- Client solicita ao host criar monitor virtual.
- Host confirma criacao.
- Client pede stream para aquele monitor.
- Host inicia somente esse stream.

Sem iniciar 3 monitores/streams automaticamente.

## Estado atual

### Ja implementado

- Host CLI e backend de monitor virtual (`xrandr --setmonitor/--delmonitor`).
- Persistencia local de monitores no host (`~/.config/parallax/virtual_displays.conf`).
- Protocolo host com mensagens:
  - `ListDisplays` / `AddVirtualDisplay` / `RemoveVirtualDisplay`
  - `ListStreams` / `SetStreamConfig`

### Ainda faltando para ponta a ponta

- Remocao sem impacto entre streams ainda depende de pipelines isolados no host.
- Host ainda opera stream real unico (`stream_id=1`), sem pipelines 1..3 independentes.

## Fluxo alvo (sequencial)

### Adicionar monitor

1. Client envia `ListDisplays` para sincronizar estado.
2. Usuario toca `+1`.
3. Client define novo `display_id` (ex.: `prlx-v2`) e geometria.
4. Client envia `AddVirtualDisplay`.
5. Host responde `DisplayOpAck` ou `Error`.
6. Client chama `ListDisplays` novamente para confirmar.
7. Client decide `stream_id` para esse display e envia `SetStreamConfig`.
8. Client envia `StartStream` para esse `stream_id`.

### Remover monitor

1. Usuario toca `-` no painel alvo.
2. Client envia `StopStream` do `stream_id` alvo.
3. Client envia `RemoveVirtualDisplay` do `display_id` alvo.
4. Host confirma `DisplayOpAck`.
5. Client atualiza UI local removendo painel.

## Contratos de dados (obrigatorios)

- `display_id`: identificador estavel do monitor virtual (ex.: `prlx-v1`).
- `stream_id`: identificador estavel do stream (1..3).
- Mapa persistente no host: `stream_id <-> display_id`.

## Implementacao detalhada por camada

### A. Android client - ControlClient

Checklist:

- [x] Expandir enum de `MessageType` em `ControlClient.kt` com tipos 0x14..0x17 e 0x30..0x34.
- [x] Implementar `listDisplays()` retornando estrutura parseada (nao string crua).
- [x] Implementar `addVirtualDisplay(id, width, height, x, y)`.
- [x] Implementar `removeVirtualDisplay(id)`.
- [x] Implementar `listStreams()` parseado.
- [x] Implementar `setStreamConfig(streamId, displayId, bindAddr, targetAddr, preferVaapi)`.
- [x] Ajustar `startStream(streamId)` e `stopStream(streamId)` com payload de `stream_id`.
- [x] Tratar `Error` em todos os comandos com mensagens amigaveis para UI.

Arquivos-alvo:

- `client/app/src/main/java/com/parallax/receiver/core/control/ControlClient.kt`

### B. Android/Quest UI - Painel de monitores

Checklist:

- [x] Criar estado local de paineis (`MonitorPanelState`).
- [x] Adicionar botao `+` para solicitar novo monitor.
- [x] Adicionar botao `-` por painel para remover monitor.
- [x] Bloquear `+` quando ja houver 3 paineis ativos.
- [x] Exibir loading/erro por operacao (`adding/removing/starting/stopping`).
- [x] Sincronizar UI com `ListDisplays` e `ListStreams` apos cada operacao.
- [x] Nao reiniciar sessao inteira ao adicionar/remover 1 painel.

Arquivos-alvo provaveis:

- `client/app/src/main/java/com/parallax/receiver/presentation/ui/StreamScreen.kt`
- `client/app/src/main/java/com/parallax/receiver/presentation/vm/StreamViewModel.kt`
- `client/app/src/main/java/com/parallax/receiver/domain/service/StreamSessionService.kt`

### C. Host daemon - Multi-stream real

Checklist:

- [x] Introduzir estrutura de stream registry (ate 3 slots).
- [x] Mapear `stream_id -> StreamConfig` com `display_id` associado.
- [x] Criar pipeline por stream (captura/encode/udp) isolado.
- [x] Implementar start/stop por stream sem afetar os demais.
- [x] Em `SetStreamConfig`, validar `display_id` existente.
- [x] Em `RemoveVirtualDisplay`, impedir remocao se stream ativo (ou parar automaticamente).
- [x] Expor `ListStreams` com metadados completos (`display_id,width,height,fps,bitrate,running`).

Arquivos-alvo provaveis:

- `host/src/control/server.rs`
- `host/src/control/session.rs`
- `host/src/stream/*`
- `host/src/capture/*`

### D. Regras de sequenciamento (nao pular)

Checklist:

- [x] Nunca iniciar stream antes de `DisplayOpAck` do `AddVirtualDisplay`.
- [x] Nunca remover display sem parar stream correspondente antes.
- [x] Sempre reconciliar estado apos comando mutavel com `ListDisplays`/`ListStreams`.
- [x] Se host responder erro, UI deve manter estado anterior e mostrar acao de retry.

## API de alto nivel sugerida no client

- `requestAddMonitor()`
- `requestRemoveMonitor(displayId)`
- `requestStartStream(streamId)`
- `requestStopStream(streamId)`
- `requestStartMonitor(displayId)`
- `requestStopMonitor(displayId)`
- `refreshTopology()`

## Criterios de aceite finais

- [ ] Usuario adiciona 1, 2 ou 3 monitores sem reiniciar sessao.
- [ ] Cada monitor consome recurso apenas quando solicitado.
- [ ] Remover monitor libera recurso correspondente no host.
- [ ] Falha em um monitor nao derruba os outros.
- [ ] UI Linux continua apenas com lifecycle/observabilidade.
