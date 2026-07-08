# Mapeamento dos Prototipos para a Arquitetura do Atlas

## Objetivo

Este documento relaciona os prototipos iniciais do projeto com a arquitetura-alvo definida em `docs/especificacoes.md`, mostrando como os algoritmos existentes devem evoluir para a aplicacao final em Rust.

## Visao Geral

O projeto nasceu de um fluxo em duas etapas:

1. `mapping_structs.rs` faz a varredura recursiva do diretorio, monta a arvore em memoria, calcula metricas e gera um relatorio Markdown.
2. `generate_structs.ps1` le esse Markdown, reconstrói a arvore, transforma em JSON e gera um HTML interativo standalone.

A arquitetura nova do `atlas` elimina essa conversao indireta via Markdown e passa a operar sobre um modelo estruturado em memoria como fonte unica de verdade.

## Fluxo Antigo

```text
Diretorio raiz
  -> Rust faz varredura
  -> Gera Markdown
  -> PowerShell le Markdown
  -> Reconstrói arvore
  -> Gera JSON embutido
  -> Gera HTML interativo
```

## Fluxo Novo

```text
Diretorio raiz
  -> CLI Rust
  -> Scanner recursivo
  -> Modelo estruturado em memoria
  -> Metricas agregadas
  -> Servidor local
  -> Interface HTML
  -> Exportacoes HTML, JSON e Markdown
```

## Mapeamento Direto: Prototipo Antigo -> Arquitetura Nova

### 1. Estrutura de nos

Prototipo antigo:
- `FileNode::File`
- `FileNode::Directory`

Destino na arquitetura nova:
- `src/scanner/node.rs`

Responsabilidade futura:
- Definir o modelo serializavel da arvore.
- Representar arquivo e diretorio de forma estavel.
- Servir como base para API, HTML e exportacoes.

Observacao:
- Esse modelo deixa de ser apenas interno ao script e passa a ser o contrato central da aplicacao.

### 2. Metricas do acervo

Prototipo antigo:
- `RelatorioMetricas`
- `total_arquivos`
- `total_diretorios`
- `por_extensao`

Destino na arquitetura nova:
- `src/scanner/metrics.rs`

Responsabilidade futura:
- Consolidar contagens totais.
- Agrupar arquivos por extensao.
- Permitir inclusao futura de metricas adicionais, como itens ignorados e erros de leitura.

Observacao:
- A especificacao sugere que esse resumo componha a resposta do modelo JSON interno e a interface web.

### 3. Algoritmo de varredura recursiva

Prototipo antigo:
- `mapear_diretorio(path, metricas)`

Destino na arquitetura nova:
- `src/scanner/walker.rs`

Responsabilidade futura:
- Receber o diretorio raiz informado pela CLI.
- Percorrer diretorios e arquivos recursivamente.
- Alimentar a arvore e as metricas.
- Ordenar diretorios antes de arquivos.
- Lidar com erros parciais sem interromper todo o processo.

Observacao:
- Esse e o nucleo logico mais importante herdado do prototipo Rust.

### 4. Geracao de Markdown

Prototipo antigo:
- `imprimir_markdown_arquivo`
- escrita direta em `estrutura_diretorios.md`

Destino na arquitetura nova:
- `src/render/markdown.rs`

Responsabilidade futura:
- Gerar Markdown a partir do modelo estruturado.
- Nao usar Markdown como fonte intermediaria para a UI.
- Usar Markdown apenas como artefato final de exportacao.

Observacao:
- Na arquitetura nova, o Markdown deixa de ser insumo e passa a ser apenas saida.

### 5. Reconstrucao da arvore a partir do Markdown

Prototipo antigo:
- parser do PowerShell baseado em indentacao e regex

Destino na arquitetura nova:
- removido da arquitetura principal

Responsabilidade futura:
- nao deve existir no fluxo principal

Motivo:
- a arvore ja existira em memoria em formato estruturado
- reconstruir estrutura a partir de texto formatado e fragil e desnecessario

Observacao:
- essa e uma das principais simplificacoes da nova arquitetura

### 6. JSON embutido no HTML

Prototipo antigo:
- `$treeJson = (...) | ConvertTo-Json`
- injecao em `<script id="tree-data" type="application/json">`

Destino na arquitetura nova:
- `src/render/json.rs`
- rota `GET /api/tree`

Responsabilidade futura:
- serializar a arvore diretamente com `serde` e `serde_json`
- expor JSON via API e download
- alimentar a interface sem precisar reconstruir os dados

Observacao:
- o JSON deixa de ser gerado por PowerShell e passa a ser nativo do backend Rust

### 7. Interface HTML interativa

Prototipo antigo:
- HTML standalone gerado em PowerShell
- CSS embutido
- JavaScript embutido com arvore expansivel, busca e contadores

Destino na arquitetura nova:
- `src/templates/index.html`
- `src/assets/style.css`
- `src/assets/app.js`
- `src/render/html.rs`

Responsabilidade futura:
- renderizar a interface principal do live server
- manter recursos ja validados no prototipo:
  - busca
  - expandir tudo
  - recolher tudo
  - navegacao hierarquica
  - indicadores visuais de arquivo e diretorio

Observacao:
- o prototipo de UI ja provou boa direcao visual e funcional
- a nova arquitetura deve reaproveitar essa logica, desacoplando-a do PowerShell

### 8. Estrategias de performance na UI

Prototipo antigo:
- renderizacao tardia dos filhos
- expansao em lotes com `requestAnimationFrame`
- filtro preservando contexto hierarquico

Destino na arquitetura nova:
- `src/assets/app.js`

Responsabilidade futura:
- manter a navegacao viavel para arvores grandes
- evitar renderizacao completa antecipada quando desnecessaria
- preservar UX aceitavel em acervos extensos

Observacao:
- isso mostra que o projeto ja nasceu com preocupacao real de escala

### 9. Caminho fixo e execucao manual

Prototipo antigo:
- caminho raiz hardcoded no Rust
- fluxo dependente de arquivos locais e execucao sequencial manual

Destino na arquitetura nova:
- `src/cli.rs`
- comando `atlas serve --input ... --port ... --open`

Responsabilidade futura:
- transformar o experimento em ferramenta utilizavel
- validar entrada
- configurar porta
- abrir navegador opcionalmente

Observacao:
- esse e o ponto onde o prototipo vira produto

## O que deve ser preservado dos prototipos

Os prototipos ja validaram elementos importantes que devem ser mantidos:

1. Modelo em arvore com separacao clara entre diretorios e arquivos.
2. Contagem total de arquivos e diretorios.
3. Agrupamento por extensao.
4. Ordenacao consistente dos itens.
5. Geracao de visualizacao interativa.
6. Busca textual sobre a arvore.
7. Expansao e recolhimento de nos.
8. Capacidade de gerar um artefato HTML compartilhavel.

## O que deve mudar na implementacao nova

A nova arquitetura deve substituir:

1. Caminho fixo por argumentos de CLI.
2. Markdown como etapa intermediaria por modelo estruturado em memoria.
3. Parser em PowerShell por serializacao nativa em Rust.
4. Geracao desacoplada de artefatos por um servidor local com rotas.
5. Fluxo manual por comando unico e reprodutivel.

## Leitura arquitetural consolidada

Os prototipos nao devem ser vistos como codigo descartavel, mas como prova de conceito de dois blocos centrais:

1. Scanner e metricas.
2. Visualizacao interativa.

A evolucao do `atlas` consiste em unificar esses blocos em uma aplicacao Rust unica, modular e orientada a um modelo central compartilhado entre:

- scanner
- metricas
- servidor
- interface
- exportacoes

## Conclusao

O projeto nasceu de um pipeline experimental:

```text
Rust -> Markdown -> PowerShell -> JSON -> HTML
```

A arquitetura-alvo transforma esse pipeline em:

```text
Rust CLI -> arvore estruturada -> servidor local -> UI + exportacoes
```

Essa mudanca reduz fragilidade, elimina conversoes intermediarias e prepara o `atlas` para crescer como ferramenta real, mantendo a logica e os comportamentos que ja se mostraram uteis nos prototipos iniciais.
