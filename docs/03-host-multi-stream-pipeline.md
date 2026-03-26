# 03 - Host Multi-Stream Pipeline

## Objetivo

Implementar pipeline de ate 3 streams simultaneos (captura + encode + UDP).

## Checklist

- [ ] Introduzir `stream_id` em toda a cadeia de envio do host.
- [ ] Instanciar pipeline por display selecionado (1..3).
- [ ] Isolar estado de cada stream (running, erro, bitrate, fps).
- [ ] Definir limites por stream (resolucao max, fps max, bitrate max).
- [ ] Implementar start/stop individual por stream.
- [ ] Implementar start/stop em lote (todos os streams ativos).
- [ ] Expor status por stream na UI do host.
- [ ] Garantir que falha de 1 stream nao derrube os demais.

## Arquivos/areas provaveis

- `host/src/stream/*`
- `host/src/net/*`
- `host/src/control/session.rs`
- `host/src/bin/prlx-host-ui/*`

## Criterios de aceite

- [ ] 3 streams iniciam sem crash.
- [ ] Stream 1/2/3 podem parar e voltar de forma independente.
- [ ] Logs identificam stream por `stream_id`.
