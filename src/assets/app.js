const treeData = JSON.parse(document.getElementById('tree-data').textContent);
const treeRoot = document.getElementById('treeRoot');
const searchInput = document.getElementById('search');
const initialNodes = Array.isArray(treeData) ? treeData : [treeData];

function formatCount(count) {
  return new Intl.NumberFormat('pt-BR').format(count);
}

function isDirectory(node) {
  return node && node.type === 'directory';
}

function getExtension(node) {
  if (node && typeof node.extension === 'string' && node.extension.length > 1) {
    return node.extension.slice(1).toLowerCase();
  }

  const match = /\.([a-z0-9]+)$/i.exec(node && node.name ? node.name : '');
  return match ? match[1].toLowerCase() : 'other';
}

function childCounts(node) {
  if (!node.children) return { folders: 0, files: 0 };
  let folders = 0;
  let files = 0;

  for (const child of node.children) {
    if (isDirectory(child)) {
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

  const extension = getExtension(node);
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
    if (isDirectory(node)) {
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
    if (!isDirectory(node)) {
      if (selfMatch) {
        filtered.push(node);
      }
      continue;
    }

    const childMatches = filterData(node.children || [], query);
    if (selfMatch || childMatches.length > 0) {
      filtered.push({
        type: 'directory',
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
