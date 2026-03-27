# 01 - Requirements and Architecture

## Objetivo

Definir escopo tecnico para suportar ate 3 telas independentes no Quest.

## Checklist

- [ ] Validar oficialmente o contrato de responsabilidades definido em `00-control-responsibilities.md`.
- [ ] Definir meta de produto (MVP 2 telas, alvo final 3 telas).
- [ ] Definir requisito de latencia alvo (ex.: <= 60ms p95 por tela).
- [ ] Definir resolucao/fps por perfil (alta, media, economia).
- [ ] Escolher estrategia de displays no Linux (outputs virtuais vs captura de monitores reais).
- [ ] Definir modelo de stream: 1 stream por tela (recomendado).
- [ ] Definir contrato de compatibilidade de protocolo (versionamento).
- [ ] Definir comportamento de fallback (quando so 1 ou 2 telas estiverem ativas).
- [ ] Definir matriz de testes (GPU AMD/Intel/NVIDIA, Quest 2/3).

## Entregaveis

- Documento de responsabilidades aprovado (client vs host-ui vs host-daemon).
- Documento de arquitetura (fluxo host -> protocolo -> cliente XR).
- Tabela de capacidades por dispositivo.
- Riscos tecnicos priorizados.

## Criterios de aceite

- [ ] Existe diagrama fim-a-fim com componentes e responsabilidades.
- [ ] Existe decisao formal de tecnologia para XR no cliente.
- [ ] Existe plano de rollout incremental (feature flags).
