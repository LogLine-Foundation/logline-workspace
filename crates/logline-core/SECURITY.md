# Segurança

Agradecemos relatos responsáveis — segurança é pilar do LogLine.

## Como reportar

- **Email:** security@logline.foundation
- **GitHub Security Advisory:** abra um *private report* no repositório

**Por favor, não abra issues públicas** para vulnerabilidades. Descreva:
- Versão afetada (`logline-core` vX.Y.Z)
- Ambiente (OS, toolchain)
- Passos para reproduzir
- Impacto esperado

Responderemos em até 5 dias úteis com:
- Confirmação de recebimento
- Classificação de severidade
- Próximos passos / timeline de correção

## Escopo

- Integridade do lifecycle (transições inválidas)
- Quebra de invariants sem erro
- Problemas de serialização que afetem canonicidade/assinatura
- Panics inesperados em inputs válidos
- Vazamento de memória relevante (em cenários realistas)

## Versionamento e patches

- Patches são lançados como `x.y.z` (patch/minor)
- Notas de segurança serão publicadas no release
