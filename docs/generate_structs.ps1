$ErrorActionPreference = 'Stop'

$sourcePath = Join-Path $PSScriptRoot 'estrutura_diretorios.md'
$outputPath = Join-Path $PSScriptRoot 'estrutura_diretorios_completa_interativo.html'

if (-not (Test-Path -LiteralPath $sourcePath)) {
  throw "Arquivo fonte nao encontrado: $sourcePath"
}

$lines = [System.IO.File]::ReadAllLines($sourcePath)
$stack = New-Object System.Collections.ArrayList
$root = [ordered]@{
  type = 'folder'
  name = 'D:/'
  children = New-Object System.Collections.ArrayList
}
[void]$stack.Add([pscustomobject]@{ indent = -1; node = $root })

$folderCount = 0
$fileCount = 0

foreach ($line in $lines) {
  if ($line -match '^(\s*)\*\s+\S+\s+\*\*(.+?)\*\*$') {
    $indent = $matches[1].Length / 2
    $name = $matches[2]
    if ($name -eq 'D:/') { continue }

    while ($stack.Count -gt 0 -and $stack[$stack.Count - 1].indent -ge $indent) {
      $stack.RemoveAt($stack.Count - 1)
    }

    $node = [ordered]@{
      type = 'folder'
      name = $name
      children = New-Object System.Collections.ArrayList
    }
    $parent = $stack[$stack.Count - 1].node
    [void]$parent.children.Add($node)
    [void]$stack.Add([pscustomobject]@{ indent = $indent; node = $node })
    $folderCount++
    continue
  }

  if ($line -match '^(\s*)\*\s+\S+\s+(.+)$') {
    $name = $matches[2]
    $parent = $stack[$stack.Count - 1].node
    [void]$parent.children.Add([ordered]@{
      type = 'file'
      name = $name
    })
    $fileCount++
  }
}

$treeJson = ($root.children | ConvertTo-Json -Depth 100 -Compress)

$html = @'
<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Estrutura Completa do Diretorio</title>
  <style>
    :root {
      --bg: #0b1020;
      --panel: #121a2d;
      --panel-2: #17233b;
      --line: #2c3a57;
      --text: #e6edf7;
      --muted: #9fb0cb;
      --accent: #5ea0ff;
      --folder: #ffd166;
      --file: #8ecae6;
      --chip: #1f2a44;
      --ok: #7bd389;
    }

    * { box-sizing: border-box; }

    html, body {
      margin: 0;
      min-height: 100%;
      background: linear-gradient(180deg, #0a0f1d 0%, #0d1324 100%);
      color: var(--text);
      font-family: "Segoe UI", Arial, Helvetica, sans-serif;
    }

    body {
      padding: 24px;
    }

    .layout {
      max-width: 1440px;
      margin: 0 auto;
      display: grid;
      grid-template-columns: 320px minmax(0, 1fr);
      gap: 20px;
    }

    .panel {
      background: rgba(18, 26, 45, 0.94);
      border: 1px solid var(--line);
      border-radius: 18px;
      box-shadow: 0 18px 50px rgba(0, 0, 0, 0.28);
      backdrop-filter: blur(8px);
    }

    .sidebar {
      position: sticky;
      top: 24px;
      align-self: start;
      padding: 20px;
    }

    .content {
      padding: 20px;
      min-width: 0;
    }

    h1 {
      margin: 0 0 10px 0;
      font-size: 24px;
      letter-spacing: 0.2px;
    }

    .subtitle {
      margin: 0 0 16px 0;
      color: var(--muted);
      line-height: 1.45;
      font-size: 14px;
    }

    .stats {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 12px;
      margin: 18px 0;
    }

    .stat-card {
      padding: 14px;
      border: 1px solid var(--line);
      border-radius: 14px;
      background: linear-gradient(180deg, rgba(23, 35, 59, 0.95), rgba(17, 25, 41, 0.95));
    }

    .stat-label {
      display: block;
      color: var(--muted);
      font-size: 12px;
      text-transform: uppercase;
      letter-spacing: 0.8px;
      margin-bottom: 4px;
    }

    .stat-value {
      font-size: 24px;
      font-weight: 700;
    }

    .controls {
      display: flex;
      flex-wrap: wrap;
      gap: 10px;
      margin: 18px 0;
    }

    .btn,
    .search {
      border-radius: 12px;
      border: 1px solid var(--line);
      background: var(--panel-2);
      color: var(--text);
      font-size: 14px;
    }

    .btn {
      padding: 10px 14px;
      cursor: pointer;
      transition: 0.18s ease;
    }

    .btn:hover {
      border-color: var(--accent);
      color: #fff;
      transform: translateY(-1px);
    }

    .search {
      width: 100%;
      padding: 11px 12px;
      outline: none;
    }

    .search:focus {
      border-color: var(--accent);
      box-shadow: 0 0 0 3px rgba(94, 160, 255, 0.18);
    }

    .legend {
      display: flex;
      flex-wrap: wrap;
      gap: 10px;
      margin-top: 16px;
    }

    .chip {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      padding: 7px 10px;
      border: 1px solid var(--line);
      border-radius: 999px;
      background: var(--chip);
      color: var(--muted);
      font-size: 12px;
    }

    .dot {
      width: 10px;
      height: 10px;
      border-radius: 50%;
      display: inline-block;
    }

    .dot.folder { background: var(--folder); }
    .dot.file { background: var(--file); }

    .tree-toolbar {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 10px;
      margin-bottom: 14px;
      padding-bottom: 12px;
      border-bottom: 1px solid var(--line);
    }

    .tree-title {
      font-size: 16px;
      font-weight: 700;
    }

    .tree-status {
      color: var(--ok);
      font-size: 12px;
      text-transform: uppercase;
      letter-spacing: 0.8px;
    }

    .tree-root,
    .tree-root ul {
      list-style: none;
      margin: 0;
      padding-left: 18px;
    }

    .tree-root {
      padding-left: 0;
      min-width: max-content;
    }

    .tree-node {
      position: relative;
      margin: 2px 0;
    }

    .tree-node::before {
      content: "";
      position: absolute;
      left: -10px;
      top: 0;
      bottom: 0;
      border-left: 1px dashed rgba(159, 176, 203, 0.22);
    }

    .tree-root > .tree-node::before {
      display: none;
    }

    details.tree-folder {
      border-radius: 10px;
    }

    details.tree-folder[open] > summary {
      background: rgba(94, 160, 255, 0.08);
    }

    summary,
    .tree-file {
      display: flex;
      align-items: center;
      gap: 10px;
      padding: 7px 10px;
      border-radius: 10px;
      color: var(--text);
      line-height: 1.35;
      word-break: break-word;
    }

    summary {
      cursor: pointer;
      list-style: none;
      transition: background 0.16s ease;
    }

    summary:hover,
    .tree-file:hover {
      background: rgba(255, 255, 255, 0.04);
    }

    summary::-webkit-details-marker {
      display: none;
    }

    .caret {
      width: 10px;
      height: 10px;
      flex: 0 0 auto;
      position: relative;
      transition: transform 0.16s ease;
    }

    .caret::before {
      content: "";
      position: absolute;
      left: 1px;
      top: 1px;
      width: 6px;
      height: 6px;
      border-right: 2px solid var(--muted);
      border-bottom: 2px solid var(--muted);
      transform: rotate(-45deg);
      transform-origin: center;
    }

    details[open] > summary .caret {
      transform: rotate(90deg);
    }

    .icon {
      flex: 0 0 auto;
      width: 16px;
      height: 14px;
      text-align: center;
      position: relative;
    }

    .icon.folder::before {
      content: "";
      position: absolute;
      left: 1px;
      top: 4px;
      width: 14px;
      height: 9px;
      background: linear-gradient(180deg, #ffd978 0%, #e4b63f 100%);
      border: 1px solid rgba(0, 0, 0, 0.28);
      border-radius: 2px;
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.22);
    }

    .icon.folder::after {
      content: "";
      position: absolute;
      left: 2px;
      top: 1px;
      width: 7px;
      height: 5px;
      background: linear-gradient(180deg, #ffe39b 0%, #efc55a 100%);
      border: 1px solid rgba(0, 0, 0, 0.22);
      border-bottom: 0;
      border-radius: 2px 2px 0 0;
    }

    .icon.file::before {
      content: "";
      position: absolute;
      left: 3px;
      top: 1px;
      width: 10px;
      height: 12px;
      background: linear-gradient(180deg, #dff4ff 0%, #93caec 100%);
      border: 1px solid rgba(0, 0, 0, 0.28);
      border-radius: 2px;
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.28);
    }

    .icon.file::after {
      content: "";
      position: absolute;
      right: 3px;
      top: 1px;
      width: 0;
      height: 0;
      border-left: 4px solid transparent;
      border-top: 4px solid #ffffff;
    }

    .name {
      min-width: 0;
      overflow-wrap: anywhere;
      font-size: 13px;
    }

    .meta {
      margin-left: auto;
      color: var(--muted);
      font-size: 11px;
      flex: 0 0 auto;
    }

    .file-badge {
      margin-left: 8px;
      padding: 2px 7px;
      border-radius: 999px;
      font-size: 10px;
      font-weight: 700;
      letter-spacing: 0.5px;
      text-transform: uppercase;
      border: 1px solid transparent;
      flex: 0 0 auto;
    }

    .file-badge.pdf {
      background: rgba(239, 68, 68, 0.14);
      color: #fca5a5;
      border-color: rgba(239, 68, 68, 0.35);
    }

    .file-badge.jpg,
    .file-badge.jpeg {
      background: rgba(34, 197, 94, 0.14);
      color: #86efac;
      border-color: rgba(34, 197, 94, 0.35);
    }

    .file-badge.tif,
    .file-badge.tiff {
      background: rgba(168, 85, 247, 0.14);
      color: #d8b4fe;
      border-color: rgba(168, 85, 247, 0.35);
    }

    .file-badge.other {
      background: rgba(148, 163, 184, 0.12);
      color: #cbd5e1;
      border-color: rgba(148, 163, 184, 0.25);
    }

    .loading-badge {
      margin-left: 8px;
      padding: 2px 8px;
      border: 1px solid rgba(94, 160, 255, 0.35);
      border-radius: 999px;
      color: var(--accent);
      background: rgba(94, 160, 255, 0.08);
      font-size: 10px;
      text-transform: uppercase;
      letter-spacing: 0.6px;
      flex: 0 0 auto;
    }

    .loading-badge.hidden {
      display: none !important;
    }

    .tree-wrap {
      overflow: auto;
      max-height: calc(100vh - 130px);
      padding-right: 8px;
    }

    .hidden {
      display: none !important;
    }

    .footer-note {
      margin-top: 16px;
      color: var(--muted);
      font-size: 12px;
      line-height: 1.5;
    }

    @media (max-width: 1080px) {
      .layout {
        grid-template-columns: 1fr;
      }

      .sidebar {
        position: static;
      }

      .tree-wrap {
        max-height: none;
      }
    }
  </style>
</head>
<body>
  <div class="layout">
    <aside class="panel sidebar">
      <h1>Estrutura Completa</h1>
      <p class="subtitle">Visualizacao tecnica interativa da arvore de diretorios e arquivos mapeada em <code>estrutura_diretorios_completa.md</code>.</p>

      <input id="search" class="search" type="search" placeholder="Buscar pasta ou arquivo...">

      <div class="controls">
        <button class="btn" id="expandAll" type="button">Expandir tudo</button>
        <button class="btn" id="collapseAll" type="button">Recolher tudo</button>
      </div>

      <div class="stats">
        <div class="stat-card">
          <span class="stat-label">Pastas</span>
          <span class="stat-value">__FOLDER_COUNT__</span>
        </div>
        <div class="stat-card">
          <span class="stat-label">Arquivos</span>
          <span class="stat-value">__FILE_COUNT__</span>
        </div>
      </div>

      <div class="legend">
        <span class="chip"><span class="dot folder"></span>Pastas</span>
        <span class="chip"><span class="dot file"></span>Arquivos</span>
      </div>

      <p class="footer-note">A arvore e navegavel por niveis. Use a busca para filtrar nomes e localizar rapidamente projetos, pastas tecnicas ou arquivos especificos.</p>
    </aside>

    <main class="panel content">
      <div class="tree-toolbar">
        <div>
          <div class="tree-title">Mapeamento do Diretorio <code>D:/</code></div>
        </div>
        <div class="tree-status">Interativo</div>
      </div>

      <div class="tree-wrap">
        <ul id="treeRoot" class="tree-root"></ul>
      </div>
    </main>
  </div>

  <script id="tree-data" type="application/json">__TREE_JSON__</script>
  <script>
    const treeData = JSON.parse(document.getElementById('tree-data').textContent);
    const treeRoot = document.getElementById('treeRoot');
    const searchInput = document.getElementById('search');
    const initialNodes = Array.isArray(treeData) ? treeData : [treeData];

    function formatCount(count) {
      return new Intl.NumberFormat('pt-BR').format(count);
    }

    function getExtension(name) {
      const match = /\.([a-z0-9]+)$/i.exec(name || '');
      return match ? match[1].toLowerCase() : 'other';
    }

    function childCounts(node) {
      if (!node.children) return { folders: 0, files: 0 };
      let folders = 0;
      let files = 0;
      for (const child of node.children) {
        if (child.type === 'folder') {
          folders += 1;
        } else {
          files += 1;
        }
      }
      return { folders, files };
    }

    function buildFolderNode(node) {
      const li = document.createElement('li');
      li.className = 'tree-node';
      li.dataset.name = (node.name || '').toLowerCase();
      li.dataset.type = node.type;

      const details = document.createElement('details');
      details.className = 'tree-folder';

      const summary = document.createElement('summary');

      const caret = document.createElement('span');
      caret.className = 'caret';

      const icon = document.createElement('span');
      icon.className = 'icon folder';

      const name = document.createElement('span');
      name.className = 'name';
      name.textContent = node.name;

      const meta = document.createElement('span');
      meta.className = 'meta';
      const counts = childCounts(node);
      meta.textContent = `${formatCount(counts.folders)} pastas | ${formatCount(counts.files)} arquivos`;

      const loadingBadge = document.createElement('span');
      loadingBadge.className = 'loading-badge hidden';
      loadingBadge.textContent = 'carregando';

      summary.append(caret, icon, name, loadingBadge, meta);
      details.appendChild(summary);

      const ul = document.createElement('ul');
      details.appendChild(ul);
      details._nodeData = node;
      details._childrenRendered = false;
      details._loadingBadge = loadingBadge;
      details.addEventListener('toggle', () => {
        if (details.open && !details._childrenRendered) {
          details._loadingBadge.classList.remove('hidden');
          requestAnimationFrame(() => {
            renderNodes(node.children || [], ul);
            details._childrenRendered = true;
            details._loadingBadge.classList.add('hidden');
          });
        }
      });

      li.appendChild(details);
      return li;
    }

    function buildFileNode(node) {
      const li = document.createElement('li');
      li.className = 'tree-node';
      li.dataset.name = (node.name || '').toLowerCase();
      li.dataset.type = node.type;

      const file = document.createElement('div');
      file.className = 'tree-file';

      const spacer = document.createElement('span');
      spacer.className = 'caret';
      spacer.textContent = '';

      const icon = document.createElement('span');
      icon.className = 'icon file';

      const name = document.createElement('span');
      name.className = 'name';
      name.textContent = node.name;

      const extension = getExtension(node.name);
      const badge = document.createElement('span');
      badge.className = `file-badge ${extension}`;
      badge.textContent = extension === 'other' ? 'ARQ' : extension;

      file.append(spacer, icon, name, badge);
      li.appendChild(file);
      return li;
    }

    function renderNodes(nodes, parent) {
      const fragment = document.createDocumentFragment();
      for (const node of nodes) {
        if (node.type === 'folder') {
          fragment.appendChild(buildFolderNode(node));
        } else {
          fragment.appendChild(buildFileNode(node));
        }
      }
      parent.appendChild(fragment);
    }

    function renderChildrenIfNeeded(details) {
      if (!details._childrenRendered) {
        const ul = details.querySelector(':scope > ul');
        if (details._loadingBadge) {
          details._loadingBadge.classList.remove('hidden');
        }
        renderNodes(details._nodeData.children || [], ul);
        details._childrenRendered = true;
        if (details._loadingBadge) {
          details._loadingBadge.classList.add('hidden');
        }
      }
    }

    function setAllFolders(open) {
      const allFolders = Array.from(document.querySelectorAll('details.tree-folder'));
      if (open) {
        let index = 0;
        function batchOpen() {
          const chunk = allFolders.slice(index, index + 150);
          for (const details of chunk) {
            renderChildrenIfNeeded(details);
            details.open = true;
          }
          index += chunk.length;
          if (index < allFolders.length) {
            requestAnimationFrame(batchOpen);
          }
        }
        batchOpen();
      } else {
        allFolders.forEach((details) => {
          details.open = false;
        });
      }
    }

    function filterData(nodes, query) {
      const filtered = [];
      for (const node of nodes) {
        const selfMatch = (node.name || '').toLowerCase().includes(query);
        if (node.type === 'file') {
          if (selfMatch) {
            filtered.push(node);
          }
          continue;
        }

        const childMatches = filterData(node.children || [], query);
        if (selfMatch || childMatches.length > 0) {
          filtered.push({
            type: 'folder',
            name: node.name,
            children: selfMatch ? (node.children || []) : childMatches
          });
        }
      }
      return filtered;
    }

    function rerender(nodes) {
      treeRoot.innerHTML = '';
      renderNodes(nodes, treeRoot);
    }

    function filterTree() {
      const query = searchInput.value.trim().toLowerCase();
      if (!query) {
        rerender(initialNodes);
        return;
      }

      const filtered = filterData(initialNodes, query);
      rerender(filtered);
      document.querySelectorAll('details.tree-folder').forEach((details) => {
        renderChildrenIfNeeded(details);
        details.open = true;
      });
    }

    rerender(initialNodes);

    document.getElementById('expandAll').addEventListener('click', () => setAllFolders(true));
    document.getElementById('collapseAll').addEventListener('click', () => setAllFolders(false));
    searchInput.addEventListener('input', filterTree);
  </script>
</body>
</html>
'@

$html = $html.Replace('__FOLDER_COUNT__', [string]$folderCount)
$html = $html.Replace('__FILE_COUNT__', [string]$fileCount)
$html = $html.Replace('__TREE_JSON__', $treeJson)

[System.IO.File]::WriteAllText($outputPath, $html, [System.Text.Encoding]::UTF8)
