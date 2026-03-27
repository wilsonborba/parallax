# 06 - Input Focus and UX

## Objetivo

Definir interacao para escolher painel ativo e enviar input corretamente ao host.

## Checklist

- [ ] Definir modelo de foco: 1 painel ativo por vez.
- [ ] Definir UX de add/remove painel no client (sem dependencia de UI Linux).
- [ ] Implementar troca de foco (gaze, controller ou hand tracking).
- [ ] Enviar eventos de teclado/mouse com `stream_id` alvo.
- [ ] Adicionar overlay de foco visivel no painel ativo.
- [ ] Implementar atalho para alternar rapido entre paineis.
- [ ] Implementar modo de bloqueio de layout para evitar movimento acidental.
- [ ] Definir UX para painel desconectado/degradado.
- [ ] Validar ergonomia (distancia, tamanho, angulo dos paineis).

## Criterios de aceite

- [ ] Nao existe ambiguidade de foco em testes de uso.
- [ ] Usuario consegue adicionar/remover paineis direto no client.
- [ ] Inputs chegam no monitor correto de forma consistente.
- [ ] Usuario consegue reorganizar paineis sem quebrar stream.
