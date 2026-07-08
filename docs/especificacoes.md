# Contexto do Projeto e Plano de Implantação/Desenvolvimento

## 1. Contexto do Projeto

O projeto tem como objetivo transformar um grande acervo documental digital, composto por pastas e arquivos distribuídos em uma estrutura hierárquica, em uma visualização técnica, organizada, navegável e consultável por meio de uma interface web local.

A proposta é consolidar a solução atual em uma aplicação única em Rust, com foco principal em uma CLI capaz de executar um live server local. Esse servidor permitirá que o usuário informe um diretório raiz, realize o mapeamento completo da estrutura e acesse uma página HTML interativa no navegador para consultar o acervo.

A solução deixa de funcionar como dois scripts separados — um para mapeamento e outro para geração de HTML — e passa a operar como uma ferramenta única, com o seguinte fluxo principal:

```text
Diretório raiz informado pelo usuário
    ↓
CLI em Rust
    ↓
Mapeamento recursivo de pastas e arquivos
    ↓
Modelo estruturado em memória
    ↓
Live server local
    ↓
Interface HTML interativa no navegador
    ↓
Downloads opcionais: HTML, JSON e Markdown
```

## 2. Objetivo Geral

Desenvolver uma ferramenta CLI em Rust para mapear, estruturar e visualizar acervos documentais digitais por meio de um servidor local, oferecendo uma interface HTML interativa com navegação hierárquica, busca, métricas e opções de exportação.

## 3. Objetivos Específicos

A solução deverá permitir:

1. Informar um diretório raiz via linha de comando.
2. Realizar a varredura recursiva de pastas e arquivos.
3. Consolidar a estrutura encontrada em um modelo de dados organizado.
4. Calcular métricas quantitativas do acervo.
5. Disponibilizar uma interface web local por meio de live server.
6. Permitir navegação interativa pela árvore de diretórios.
7. Permitir busca por nomes de arquivos e pastas.
8. Permitir expansão e recolhimento de níveis da árvore.
9. Disponibilizar download do HTML interativo standalone.
10. Disponibilizar download do JSON estruturado.
11. Disponibilizar download do relatório técnico em Markdown.
12. Registrar logs e erros encontrados durante o mapeamento.

## 4. Problema que o Projeto Resolve

Grandes acervos documentais armazenados em diretórios locais ou unidades de rede costumam ser difíceis de analisar manualmente. A navegação tradicional por pastas não oferece uma visão consolidada da estrutura, não facilita a contagem de volumes, não permite análise técnica rápida e não gera documentação reaproveitável.

O projeto resolve esse problema ao converter uma massa extensa de arquivos e pastas em uma interface visual estruturada, permitindo leitura técnica, navegação, consulta e exportação dos dados mapeados.

## 5. Escopo Inicial do Produto

O escopo inicial será concentrado na funcionalidade principal de live server.

O comando base previsto será:

```bash
atlas serve --input "D:\Acervo" --open
```

Esse comando deverá:

1. Validar o diretório informado.
2. Executar o mapeamento recursivo.
3. Calcular métricas básicas.
4. Subir um servidor local.
5. Abrir o navegador automaticamente, quando solicitado.
6. Exibir a interface HTML interativa.
7. Permitir download dos artefatos gerados.

## 6. Fora do Escopo Inicial

Para manter o MVP simples e funcional, os seguintes itens não entram na primeira versão:

1. Interface desktop com Tauri.
2. Banco SQLite para histórico.
3. Comparação entre varreduras.
4. Login ou controle de usuários.
5. Publicação em servidor externo.
6. Indexação de conteúdo interno dos arquivos.
7. Hash para identificação de duplicados.
8. Watch mode em tempo real.
9. Instalador completo.
10. Painel administrativo.

Esses itens poderão compor versões futuras.

## 7. Stack Técnica Recomendada

A stack inicial será:

```text
Rust
clap
axum
tokio
serde
serde_json
tera ou askama
tower-http
open
tracing
```

Responsabilidades principais:

```text
clap        Entrada por argumentos e subcomandos da CLI
axum        Servidor HTTP local
tokio       Runtime assíncrono
serde       Serialização dos modelos
serde_json  Geração e resposta JSON
tera/askama Renderização do HTML
tower-http  Serviço de arquivos e assets estáticos
open        Abertura automática do navegador
tracing     Logs estruturados
```

## 8. Arquitetura Proposta

A arquitetura será modular, separando varredura, modelo de dados, servidor, renderização e exportação.

```text
atlas-mapper/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── scanner/
│   │   ├── mod.rs
│   │   ├── node.rs
│   │   ├── metrics.rs
│   │   └── walker.rs
│   ├── server/
│   │   ├── mod.rs
│   │   └── routes.rs
│   ├── render/
│   │   ├── mod.rs
│   │   ├── html.rs
│   │   ├── markdown.rs
│   │   └── json.rs
│   ├── templates/
│   │   └── index.html
│   └── assets/
│       ├── style.css
│       └── app.js
└── README.md
```

## 9. Modelo de Funcionamento

O live server será o centro da experiência.

Em vez de gerar primeiro um Markdown e depois converter o Markdown para HTML, a aplicação deverá trabalhar com um modelo estruturado em memória.

Fluxo recomendado:

```text
Scanner
    ↓
Modelo de dados
    ↓
Métricas
    ↓
Servidor local
    ↓
Rotas de visualização e download
```

Rotas previstas:

```text
GET /                    Página principal HTML
GET /api/tree             Retorna árvore em JSON
GET /api/summary          Retorna métricas do acervo
GET /download/html        Baixa HTML standalone
GET /download/json        Baixa JSON estruturado
GET /download/markdown    Baixa relatório Markdown
```

## 10. Funcionalidades do MVP

### 10.1 CLI

A CLI deverá permitir:

```bash
atlas serve --input "D:\Acervo"
```

Parâmetros iniciais:

```text
--input   Caminho do diretório raiz a ser mapeado
--port    Porta do servidor local
--open    Abre automaticamente o navegador
```

Exemplo:

```bash
atlas serve --input "D:\Acervo" --port 8787 --open
```

### 10.2 Live Server

O servidor local deverá exibir o endereço no terminal:

```text
Atlas Mapper iniciado.

Origem analisada:
D:\Acervo

Servidor local:
http://127.0.0.1:8787
```

### 10.3 Interface HTML

A interface deverá conter:

1. Nome do projeto.
2. Caminho de origem analisado.
3. Data e hora do mapeamento.
4. Total de diretórios.
5. Total de arquivos.
6. Quantitativo por extensão.
7. Campo de busca.
8. Botão de expandir tudo.
9. Botão de recolher tudo.
10. Árvore interativa de pastas e arquivos.
11. Botão para baixar HTML.
12. Botão para baixar JSON.
13. Botão para baixar Markdown.

### 10.4 Exportações

A aplicação deverá disponibilizar três formatos de exportação:

```text
HTML      Visualização interativa standalone para compartilhamento
JSON      Estrutura técnica consumível por outras ferramentas
Markdown  Relatório textual técnico do acervo
```

## 11. Plano de Desenvolvimento por Etapas

## Etapa 1 — Reestruturação do Projeto Rust

Objetivo: transformar o código atual em um projeto modular e preparado para evolução.

Atividades:

1. Criar estrutura de pastas do projeto.
2. Separar o código de varredura do `main.rs`.
3. Criar módulo `scanner`.
4. Criar módulo `node`.
5. Criar módulo `metrics`.
6. Remover caminho fixo do código.
7. Preparar entrada via argumentos.
8. Manter geração de árvore em memória.
9. Manter contagem de arquivos, diretórios e extensões.

Resultado esperado:

```text
O projeto deixa de ser um script único e passa a ter uma estrutura modular em Rust.
```

Critérios de aceite:

```text
Dado que o usuário informa um diretório válido,
quando a aplicação executa a varredura,
então a estrutura deve ser mapeada em memória com arquivos, pastas e métricas.
```

## Etapa 2 — Implementação da CLI

Objetivo: permitir execução do projeto por linha de comando.

Atividades:

1. Adicionar `clap`.
2. Criar comando `serve`.
3. Criar parâmetro `--input`.
4. Criar parâmetro `--port`.
5. Criar parâmetro `--open`.
6. Validar se o diretório informado existe.
7. Exibir mensagens claras no terminal.
8. Tratar erro de caminho inválido.

Comando esperado:

```bash
atlas serve --input "D:\Acervo" --port 8787 --open
```

Critérios de aceite:

```text
Dado que o usuário informa um caminho inválido,
quando executar o comando,
então a aplicação deve exibir erro claro e não iniciar o servidor.

Dado que o usuário informa um caminho válido,
quando executar o comando,
então a aplicação deve mapear o diretório e iniciar o live server.
```

## Etapa 3 — Criação do Modelo JSON Interno

Objetivo: criar um modelo estruturado que sirva como fonte de verdade para a interface e exportações.

Atividades:

1. Adicionar `serde`.
2. Tornar os nós serializáveis.
3. Criar estrutura de resposta do mapeamento.
4. Incluir origem analisada.
5. Incluir data de geração.
6. Incluir métricas gerais.
7. Incluir árvore completa.
8. Preparar geração de JSON.

Modelo previsto:

```json
{
  "source": "D:/Acervo",
  "generated_at": "2026-07-08T09:00:00",
  "summary": {
    "total_directories": 1200,
    "total_files": 85000,
    "extensions": {
      ".pdf": 30000,
      ".docx": 12000
    }
  },
  "tree": {
    "type": "directory",
    "name": "Acervo",
    "children": []
  }
}
```

Critérios de aceite:

```text
Dado que a varredura foi concluída,
quando o modelo for serializado,
então o JSON deve representar a origem, métricas e árvore completa do acervo.
```

## Etapa 4 — Implementação do Live Server

Objetivo: subir um servidor local para disponibilizar a interface.

Atividades:

1. Adicionar `axum`.
2. Adicionar `tokio`.
3. Criar módulo `server`.
4. Criar rota `/`.
5. Criar rota `/api/tree`.
6. Criar rota `/api/summary`.
7. Configurar porta padrão.
8. Permitir porta customizada.
9. Exibir URL no terminal.
10. Abrir navegador com `--open`.

Porta padrão sugerida:

```text
8787
```

Critérios de aceite:

```text
Dado que a aplicação foi iniciada com sucesso,
quando o usuário acessar a URL local,
então a página principal deve ser exibida no navegador.

Dado que o usuário acessa /api/tree,
quando a rota é chamada,
então o servidor deve retornar a árvore em JSON.
```

## Etapa 5 — Interface HTML Interativa

Objetivo: criar a primeira versão navegável da interface.

Atividades:

1. Criar template `index.html`.
2. Criar CSS básico.
3. Criar JavaScript da árvore.
4. Renderizar diretórios e arquivos.
5. Implementar expansão e recolhimento.
6. Implementar busca por nome.
7. Exibir métricas gerais.
8. Exibir origem analisada.
9. Exibir data de geração.
10. Melhorar leitura visual da árvore.

Critérios de aceite:

```text
Dado que a página foi aberta,
quando a árvore for carregada,
então o usuário deve conseguir navegar pelas pastas e arquivos.

Dado que o usuário informa um termo no campo de busca,
quando houver correspondências,
então a interface deve destacar ou filtrar os itens encontrados.
```

## Etapa 6 — Downloads pelo Navegador

Objetivo: permitir exportação dos artefatos diretamente pela interface.

Atividades:

1. Criar rota `/download/json`.
2. Criar rota `/download/markdown`.
3. Criar rota `/download/html`.
4. Implementar geração de Markdown a partir do modelo.
5. Implementar HTML standalone.
6. Adicionar botões de download na interface.
7. Definir nomes padronizados dos arquivos.

Arquivos esperados:

```text
estrutura_diretorios.html
estrutura_diretorios.json
estrutura_diretorios.md
```

Critérios de aceite:

```text
Dado que o usuário acessa a interface,
quando clicar em Baixar JSON,
então o navegador deve baixar o arquivo JSON estruturado.

Dado que o usuário clicar em Baixar Markdown,
então o navegador deve baixar o relatório técnico em Markdown.

Dado que o usuário clicar em Baixar HTML,
então o navegador deve baixar uma versão standalone e interativa do relatório.
```

## Etapa 7 — Tratamento de Erros e Logs

Objetivo: tornar a aplicação confiável em diretórios grandes e ambientes reais.

Atividades:

1. Registrar erros de leitura.
2. Registrar diretórios sem permissão.
3. Registrar arquivos inacessíveis.
4. Evitar falha completa por erro parcial.
5. Exibir resumo de alertas na interface.
6. Exibir logs no terminal.
7. Criar métrica de itens ignorados.
8. Padronizar mensagens de erro.

Critérios de aceite:

```text
Dado que uma pasta não pode ser lida,
quando a varredura passar por ela,
então a aplicação deve registrar o erro e continuar o mapeamento.

Dado que existam erros durante a varredura,
quando a interface for aberta,
então o resumo deve indicar que houve avisos ou itens ignorados.
```

## Etapa 8 — Empacotamento Inicial

Objetivo: permitir uso da ferramenta sem depender do ambiente de desenvolvimento.

Atividades:

1. Gerar build release.
2. Criar instruções de execução.
3. Criar README.
4. Definir nome do binário.
5. Testar em Windows.
6. Testar caminhos com espaço.
7. Testar diretórios grandes.
8. Criar exemplo de execução.

Comando de build:

```bash
cargo build --release
```

Resultado esperado:

```text
target/release/atlas.exe
```

Critérios de aceite:

```text
Dado que o usuário possui o binário,
quando executar o comando atlas serve,
então a aplicação deve funcionar sem necessidade de abrir o projeto no editor.
```

## 12. Roadmap Pós-MVP

Após a primeira versão funcional, o projeto poderá evoluir para:

```text
v0.2 — Melhorias de filtro
- Filtro por extensão
- Filtro por tipo
- Filtro por tamanho
- Ordenação por nome, tipo e extensão

v0.3 — Relatórios técnicos avançados
- Pastas vazias
- Arquivos sem extensão
- Top maiores arquivos
- Top maiores diretórios
- Erros de permissão detalhados

v0.4 — Watch mode
- Atualizar visualização ao detectar mudanças
- Reexecutar mapeamento sob demanda
- Botão de atualizar acervo

v0.5 — Exportação compactada
- Download ZIP
- HTML + JSON + Markdown no mesmo pacote
- Assets organizados

v1.0 — Versão estável
- Binário versionado
- Documentação completa
- Testes automatizados
- Release para Windows
```

## 13. Critérios Gerais de Sucesso

O projeto será considerado bem-sucedido quando:

1. O usuário conseguir executar o mapeamento informando apenas o diretório raiz.
2. A aplicação abrir um link local no navegador.
3. A árvore de diretórios for apresentada de forma interativa.
4. A busca funcionar para arquivos e pastas.
5. As métricas principais forem exibidas corretamente.
6. O HTML puder ser baixado e compartilhado.
7. O JSON puder ser baixado para consumo técnico.
8. O Markdown puder ser baixado como relatório documental.
9. Erros de leitura não interromperem toda a execução.
10. O projeto estiver organizado para evolução futura.

## 14. Resumo Executivo

O projeto será evoluído para uma CLI em Rust com live server local como funcionalidade principal. A ferramenta permitirá mapear um acervo documental a partir de um diretório raiz, estruturar os dados em memória, exibir uma interface HTML interativa no navegador e disponibilizar downloads do HTML standalone, JSON estruturado e relatório Markdown.

A primeira versão deverá priorizar simplicidade, robustez e utilidade imediata: executar o comando, abrir o navegador, navegar pelo acervo e baixar os artefatos finais. Funcionalidades mais avançadas, como histórico, comparação de versões, watch mode e interface desktop, serão tratadas como evolução posterior.
