# 02 - Linux Multi-Display Foundation

## Objetivo

Preparar o host Linux para expor ate 3 saidas de desktop capturaveis.

## Checklist

- [ ] Definir backend alvo inicial: X11 (atual) com plano para Wayland.
- [ ] Implementar criacao/gestao de displays virtuais quando necessario.
- [x] Adicionar descoberta de displays disponiveis no host.
- [x] Definir identificador estavel de display (`display_id`).
- [x] Persistir configuracao de displays (arquivo de config do host).
- [x] Adicionar comando CLI para listar displays (`prlx-hostd --list-displays`).
- [x] Adicionar comando CLI para habilitar/desabilitar display virtual.
- [ ] Tratar cleanup em shutdown (sem deixar estado quebrado no X server).

## Arquivos/areas provaveis

- `host/src/cli.rs`
- `host/src/capture/*`
- `host/src/control/server.rs`
- Novo modulo sugerido: `host/src/display/*`

## Criterios de aceite

- [ ] Host lista ate 3 displays de forma consistente.
- [ ] Cada display pode ser ativado/desativado sem reiniciar maquina.
- [ ] Reinicio do host mantem configuracao esperada.

## Status da etapa

- Implementado comando `prlx-hostd --list-displays`.
- Implementado `prlx-hostd --list-virtual-displays`.
- Implementado `prlx-hostd --virtual-backend-status` para diagnostico de ambiente.
- Implementado `prlx-hostd --enable-virtual-display ...` e `--disable-virtual-display ...`.
- Persistencia implementada em `~/.config/parallax/virtual_displays.conf`.
- Aplicacao automatica dos displays virtuais persistidos ao iniciar `prlx-hostd`.
- Implementacao de virtual display usa `xrandr --setmonitor/--delmonitor` (suporte depende do ambiente X11).
