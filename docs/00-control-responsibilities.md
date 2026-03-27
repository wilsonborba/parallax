# 00 - Control Responsibilities (Source of Truth)

## Objetivo

Definir de forma explicita quem controla cada parte do sistema.

## Regras obrigatorias

- O **Client (Android/Meta Quest)** e o dono das features de sessao:
  - adicionar monitor virtual
  - remover monitor virtual
  - abrir tela/painel
  - fechar tela/painel
  - alterar layout de paineis e configuracoes de stream
- O **Host UI (Linux)** e o dono do lifecycle local do host:
  - iniciar host
  - parar host
  - mostrar estado local, QR e diagnostico
- O **Host daemon (`prlx-hostd`)** nao decide produto/UX:
  - apenas executa comandos recebidos da UI local e do Client
  - expoe estado e capacidades pelo protocolo de controle

## Fluxo de autoridade

1. Client pareia via QR.
2. Client envia comandos de features multi-monitor para o host.
3. Host daemon aplica e responde status.
4. Host UI reflete estado e permite apenas controle local de start/stop.

## Impacto nas etapas

- Etapa 03: host precisa aceitar comandos orientados a `stream_id`/`display_id` vindos do client.
- Etapa 04: protocolo deve incluir comandos de add/remove/list monitor e open/close stream.
- Etapa 06: foco/input deve ser sempre associado ao painel selecionado no client.

## Criterios de aceite

- [ ] Nenhuma acao de add/remove monitor depende da UI Linux.
- [ ] UI Linux nao implementa logica de produto do Quest; apenas lifecycle/observabilidade.
- [ ] Client consegue controlar 100% do estado de telas remotas via protocolo.
