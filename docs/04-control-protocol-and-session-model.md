# 04 - Control Protocol and Session Model

## Objetivo

Evoluir canal de controle para sessao multi-stream e configuracao dinamica.

## Checklist

- [ ] Versionar protocolo (ex.: `protocol_version=2`).
- [x] Adicionar mensagens: `ListStreams`, `StartStream`, `StopStream`, `SetStreamConfig`.
- [x] Adicionar mensagens client-driven de monitor: `AddVirtualDisplay`, `RemoveVirtualDisplay`, `ListDisplays`.
- [x] Garantir autorizacao de comando: client pareado pode controlar features de monitor/sessao.
- [x] Incluir `stream_id` em comandos e respostas.
- [ ] Incluir metadados por stream (display_id, largura, altura, fps, bitrate).
- [ ] Definir erros padronizados por stream.
- [ ] Atualizar QR/pairing para carregar capacidades multi-stream.
- [ ] Garantir backward compatibility (cliente antigo continua funcionando com 1 stream).
- [ ] Atualizar `proto/README.md` com exemplos reais de payload.

## Arquivos/areas provaveis

- `host/src/control/protocol.rs`
- `host/src/control/session.rs`
- `host/src/control/server.rs`
- `proto/README.md`

## Criterios de aceite

- [ ] Cliente recebe lista de streams sem hacks.
- [ ] Cliente consegue adicionar/remover monitor sem usar a UI Linux.
- [ ] Alteracoes de config em runtime funcionam por stream.
- [ ] Sessao reconnect preserva estado esperado.

## Status da etapa

- Implementado no host:
  - `ListStreams` (`0x14`) -> `Streams` (`0x15`)
  - `SetStreamConfig` (`0x16`) -> `StreamConfigAck` (`0x17`) ou `Error`
  - `ListDisplays` (`0x30`) -> `Displays` (`0x31`)
  - `AddVirtualDisplay` (`0x32`) -> `DisplayOpAck` (`0x34`) ou `Error`
  - `RemoveVirtualDisplay` (`0x33`) -> `DisplayOpAck` (`0x34`) ou `Error`
- Comandos acima exigem sessao pareada (`HandshakeState::Paired`).
- `stream_id` implementado inicialmente com stream unico (`stream_id=1`) para backward compatibility.
- Pendente: multi-stream real (1..3) no pipeline de streaming.

## Payloads atuais (para client)

- `ListStreams (0x14)`: payload vazio.
- `StartStream (0x10)`: payload opcional `stream_id=<id>` (ou vazio para `1`).
- `StopStream (0x11)`: payload opcional `stream_id=<id>` (ou vazio para `1`).
- `Streams (0x15)`: UTF-8 texto:
  - `protocol=2`
  - `streams:`
  - `1,<display>,<bind_addr>,<target_addr>,<prefer_vaapi>,<running>`
- `SetStreamConfig (0x16)`: UTF-8 com linhas `key=value`:
  - obrigatorio: `stream_id=1`
  - opcionais: `display=...`, `bind_addr=...`, `target_addr=...`, `prefer_vaapi=true|false`
- `StreamConfigAck (0x17)`: ack de sucesso.
