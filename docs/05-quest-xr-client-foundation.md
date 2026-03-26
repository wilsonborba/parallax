# 05 - Quest XR Client Foundation

## Objetivo

Migrar cliente para experiencia XR com ate 3 paineis (telas) no espaco.

## Checklist

- [ ] Escolher stack XR para Quest (OpenXR + engine/framework escolhido).
- [ ] Criar cena base com ancoragem espacial.
- [ ] Criar 3 superficies renderizaveis independentes (painel A/B/C).
- [ ] Associar decoder de video por `stream_id`.
- [ ] Implementar fallback para 1 painel quando apenas 1 stream existir.
- [ ] Implementar reposicionamento de painel no mundo 3D.
- [ ] Persistir layout espacial localmente no dispositivo.
- [ ] Exibir status visual por painel (connecting, streaming, degraded).

## Arquivos/areas provaveis

- `client/` (novo modulo XR ou novo app)
- Camada de networking/decoder no cliente
- Camada de UI/UX para layout espacial

## Criterios de aceite

- [ ] Usuario consegue ver 3 paineis distintos no Quest.
- [ ] Cada painel reproduz stream correto sem trocar IDs.
- [ ] Reabrir app restaura layout anterior.
