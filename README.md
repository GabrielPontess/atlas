# Atlas

Ferramenta CLI em Rust para mapear um diretorio, subir um servidor local e abrir uma interface HTML interativa para navegacao, busca e exportacao de artefatos.

## Estado Atual

O projeto ja entrega:

1. Varredura recursiva de diretorios e arquivos.
2. Metricas de diretorios, arquivos, extensoes, itens ignorados e avisos.
3. Live server local com interface HTML interativa.
4. API local para arvore e resumo.
5. Downloads de `HTML`, `JSON` e `Markdown`.
6. Tratamento de erros parciais durante a leitura do acervo.

## Requisitos

1. Rust toolchain instalado.
2. Windows suportado e validado no fluxo atual.

## Executando em Desenvolvimento

Use um diretorio local como entrada:

```bash
cargo run -- serve --input "C:\Acervo" --port 8787 --open
```

Argumentos disponiveis:

1. `--input` caminho do diretorio raiz a ser mapeado.
2. `--port` porta do servidor local. Padrao: `8787`.
3. `--open` abre o navegador automaticamente.

## Build Release

Para gerar o binario final:

```bash
cargo build --release
```

Binario esperado no Windows:

```text
target/release/atlas.exe
```

## Executando o Binario

Exemplo de uso apos o build:

```bash
target\release\atlas.exe serve --input "C:\Acervo" --port 8787 --open
```

## Rotas Disponiveis

Quando o servidor sobe, a aplicacao exibe uma URL local como:

```text
http://127.0.0.1:8787
```

Rotas atuais:

1. `/` interface HTML interativa.
2. `/api/tree` arvore em JSON.
3. `/api/summary` resumo tecnico em JSON.
4. `/download/html` exportacao de HTML standalone.
5. `/download/json` exportacao do modelo JSON.
6. `/download/markdown` exportacao do relatorio Markdown.

## Exportacoes

Arquivos gerados pelo navegador:

1. `estrutura_diretorios.html`
2. `estrutura_diretorios.json`
3. `estrutura_diretorios.md`

## Observacoes

1. Caminhos com espacos sao suportados desde que estejam entre aspas.
2. Erros parciais de leitura nao encerram todo o mapeamento; eles aparecem como avisos no terminal e na interface.
3. O HTML baixado em `/download/html` e standalone e nao depende do live server depois de salvo.
