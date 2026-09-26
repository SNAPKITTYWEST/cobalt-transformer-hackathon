// COBOL Transformer Playground - UI glue only.
// All COBOL processing happens in the Rust crate compiled to WebAssembly
// (web/wasm -> pkg/cobol_transformer_wasm.js). This file only wires controls
// to `run(operation, source, optionsJson)` and renders the JSON it returns.

import init, { run, version } from './pkg/cobol_transformer_wasm.js';

const $ = (id) => document.getElementById(id);
const els = {
  sample: $('sample-select'),
  sampleNote: $('sample-note'),
  fileName: $('file-name'),
  source: $('source'),
  gutter: $('gutter'),
  cursor: $('cursor-pos'),
  size: $('source-size'),
  op: $('operation'),
  run: $('run'),
  optPasses: $('opt-passes'),
  optFormat: $('opt-format'),
  optNormalizeFirst: $('opt-normalize-first'),
  format: $('format'),
  normalizeFirst: $('normalize-first'),
  autoRun: $('auto-run'),
  errorBox: $('error-box'),
  errorStage: $('error-stage'),
  errorPos: $('error-pos'),
  errorMsg: $('error-message'),
  errorGoto: $('error-goto'),
  diagBox: $('diag-box'),
  diagList: $('diag-list'),
  tabs: document.querySelectorAll('.view-tab'),
  output: $('output'),
  expandAll: $('expand-all'),
  collapseAll: $('collapse-all'),
  copy: $('copy-output'),
  status: $('status'),
  wasmStatus: $('wasm-status'),
  theme: $('theme-toggle'),
  debug: $('debug'),
  debugPanel: $('debug-panel'),
  debugLog: $('debug-log'),
  debugClear: $('debug-clear'),
};

// ---------------------------------------------------------------- debug
// Enabled by the "debug" checkbox or ?debug=1. Entries go to the on-page
// panel and to console.debug; the panel keeps the last 200 lines.
const DEBUG_KEY = 'cobol-playground-debug';
function readDebugPref() {
  if (new URLSearchParams(location.search).get('debug') === '1') return true;
  try { return localStorage.getItem(DEBUG_KEY) === '1'; } catch { return false; }
}
els.debug.checked = readDebugPref();
els.debugPanel.classList.toggle('hidden', !els.debug.checked);
els.debug.addEventListener('change', () => {
  els.debugPanel.classList.toggle('hidden', !els.debug.checked);
  try { localStorage.setItem(DEBUG_KEY, els.debug.checked ? '1' : '0'); } catch { /* storage unavailable */ }
  dbg('debug', els.debug.checked ? 'enabled' : 'disabled');
});
els.debugClear.addEventListener('click', () => { els.debugLog.textContent = ''; });

function dbg(event, detail) {
  if (!els.debug.checked) return;
  const time = new Date().toISOString().slice(11, 23);
  const text = detail === undefined ? '' : typeof detail === 'string' ? detail : JSON.stringify(detail);
  console.debug(`[playground] ${event}`, detail ?? '');
  const lines = (els.debugLog.textContent ? els.debugLog.textContent.split('\n') : []);
  lines.push(`${time} ${event}${text ? ' ' + text : ''}`);
  els.debugLog.textContent = lines.slice(-200).join('\n');
  els.debugLog.scrollTop = els.debugLog.scrollHeight;
}

const TAB_ON = ['bg-indigo-100', 'text-indigo-800', 'dark:bg-indigo-900/50', 'dark:text-indigo-200'];
const TAB_OFF = ['text-slate-600', 'hover:bg-slate-100', 'dark:text-slate-400', 'dark:hover:bg-slate-800'];

// Which output views make sense for each operation (first = default).
const VIEWS = {
  parse: ['raw', 'json'],
  ast: ['tree', 'raw', 'json'],
  symbols: ['tree', 'raw', 'json'],
  cfg: ['tree', 'raw', 'json'],
  tokens: ['table', 'raw', 'json'],
  generate: ['raw', 'json'],
  'round-trip': ['raw', 'json'],
  'detect-format': ['raw', 'json'],
  'normalize-format': ['raw', 'json'],
};
const FRONT_END_OPS = new Set(['parse', 'ast', 'symbols', 'cfg', 'generate', 'round-trip']);

let ready = false;
let lastResult = null;
let currentView = 'tree';
let errorLine = null;
let debounceTimer = null;

// ---------------------------------------------------------------- theme
els.theme.addEventListener('click', () => {
  const dark = document.documentElement.classList.toggle('dark');
  try { localStorage.setItem('cobol-playground-theme', dark ? 'dark' : 'light'); } catch (e) { /* storage unavailable */ }
});

// ---------------------------------------------------------------- editor
function lineCount(text) {
  return text.length === 0 ? 1 : text.split('\n').length;
}

function renderGutter() {
  const n = lineCount(els.source.value);
  const frag = document.createDocumentFragment();
  for (let i = 1; i <= n; i++) {
    const s = document.createElement('span');
    s.textContent = String(i);
    if (i === errorLine) s.className = 'font-bold text-rose-600 dark:text-rose-400';
    frag.appendChild(s);
    if (i < n) frag.appendChild(document.createTextNode('\n'));
  }
  els.gutter.replaceChildren(frag);
  els.gutter.scrollTop = els.source.scrollTop;
  els.size.textContent = `${n} line${n === 1 ? '' : 's'}, ${els.source.value.length} chars`;
}

function updateCursor() {
  const pos = els.source.selectionStart;
  const before = els.source.value.slice(0, pos);
  const line = before.split('\n').length;
  const col = pos - before.lastIndexOf('\n');
  els.cursor.textContent = `Ln ${line}, Col ${col}`;
}

function gotoLine(line, column) {
  const lines = els.source.value.split('\n');
  const l = Math.max(1, Math.min(line, lines.length));
  let offset = 0;
  for (let i = 0; i < l - 1; i++) offset += lines[i].length + 1;
  const lineLen = lines[l - 1].length;
  const c = Math.max(1, Math.min(column || 1, lineLen + 1));
  els.source.focus();
  els.source.setSelectionRange(offset + c - 1, offset + Math.min(lineLen, c));
  els.source.scrollTop = Math.max(0, (l - 4) * 20);
  els.gutter.scrollTop = els.source.scrollTop;
  updateCursor();
}

els.source.addEventListener('scroll', () => { els.gutter.scrollTop = els.source.scrollTop; });
els.source.addEventListener('input', () => { renderGutter(); updateCursor(); scheduleRun(); });
els.source.addEventListener('click', updateCursor);
els.source.addEventListener('keyup', updateCursor);
els.source.addEventListener('keydown', (e) => {
  if (e.key === 'Tab' && !e.ctrlKey && !e.metaKey && !e.altKey) {
    e.preventDefault();
    const { selectionStart: s, selectionEnd: t, value } = els.source;
    els.source.value = value.slice(0, s) + '    ' + value.slice(t);
    els.source.setSelectionRange(s + 4, s + 4);
    els.source.dispatchEvent(new Event('input'));
  }
});
// Ctrl/Cmd+Enter runs once per press: auto-repeat while held and Enter used
// to confirm an IME composition are ignored.
document.addEventListener('keydown', (e) => {
  if (e.key !== 'Enter' || !(e.ctrlKey || e.metaKey) || e.altKey || e.shiftKey) return;
  e.preventDefault();
  if (e.repeat || e.isComposing) {
    dbg('run-ignored', e.repeat ? 'Ctrl+Enter auto-repeat' : 'Enter during IME composition');
    return;
  }
  runNow('keyboard');
});

// ---------------------------------------------------------------- samples
let samples = [];

async function loadSamples() {
  try {
    const res = await fetch('./samples/samples.json');
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    samples = await res.json();
  } catch (e) {
    els.sample.replaceChildren(new Option('(samples unavailable)', ''));
    return;
  }
  const opts = [new Option('- paste your own -', '')];
  for (const s of samples) opts.push(new Option(s.label, s.id));
  els.sample.replaceChildren(...opts);
  // Optional deep link: ?sample=<id>&op=<operation>
  const params = new URLSearchParams(location.search);
  const op = params.get('op');
  if (op && VIEWS[op]) { els.op.value = op; currentView = VIEWS[op][0]; syncOptions(); }
  const wanted = params.get('sample');
  els.sample.value = samples.some((s) => s.id === wanted) ? wanted : (samples[0]?.id ?? '');
  await selectSample(els.sample.value);
}

async function selectSample(id) {
  const s = samples.find((x) => x.id === id);
  if (!s) { els.sampleNote.classList.add('hidden'); return; }
  const res = await fetch(`./samples/${s.file}`);
  const text = (await res.text()).replace(/\r\n?/g, '\n');
  els.source.value = text;
  els.source.setSelectionRange(0, 0);
  els.fileName.value = s.path.split('/').pop();
  els.source.scrollTop = 0;
  els.sampleNote.textContent = `${s.path}${s.note ? ' - ' + s.note : ''}`;
  els.sampleNote.classList.remove('hidden');
  errorLine = null;
  renderGutter();
  updateCursor();
  execute('sample');
}

els.sample.addEventListener('change', () => selectSample(els.sample.value));

// ---------------------------------------------------------------- options
function syncOptions() {
  const op = els.op.value;
  els.optPasses.classList.toggle('hidden', op !== 'generate');
  els.optNormalizeFirst.classList.toggle('hidden', !FRONT_END_OPS.has(op));
  const showFormat = op === 'normalize-format' || (FRONT_END_OPS.has(op) && els.normalizeFirst.checked);
  els.optFormat.classList.toggle('hidden', !showFormat);
  const views = VIEWS[op];
  els.tabs.forEach((t) => t.classList.toggle('hidden', !views.includes(t.dataset.view)));
  if (!views.includes(currentView)) currentView = views[0];
  const tree = currentView === 'tree';
  els.expandAll.classList.toggle('hidden', !tree);
  els.collapseAll.classList.toggle('hidden', !tree);
}

function collectOptions() {
  return {
    file_name: els.fileName.value.trim() || 'input.cob',
    passes: [...document.querySelectorAll('input[name="pass"]:checked')].map((c) => c.value),
    format: els.format.value,
    normalize_first: els.normalizeFirst.checked,
  };
}

els.op.addEventListener('change', () => { currentView = VIEWS[els.op.value][0]; syncOptions(); execute('operation'); });
for (const el of [els.format, els.normalizeFirst, ...document.querySelectorAll('input[name="pass"]')]) {
  el.addEventListener('change', () => { syncOptions(); execute('option'); });
}
els.fileName.addEventListener('change', () => execute('file-name'));
els.run.addEventListener('click', () => {
  dbg('run-click', { ready, operation: els.op.value, disabled: els.run.disabled });
  runNow('button');
});

// Explicit runs (button, Ctrl+Enter) cancel any pending run-on-edit timer,
// so typing then pressing Run executes once, not once now and again 300 ms later.
function runNow(trigger) {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
    debounceTimer = null;
    dbg('debounce-cancelled', `pending edit run replaced by ${trigger}`);
  }
  execute(trigger);
}
els.tabs.forEach((t) => t.addEventListener('click', () => { currentView = t.dataset.view; syncOptions(); renderOutput(); }));

function scheduleRun() {
  if (!els.autoRun.checked) return;
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => { debounceTimer = null; execute('edit'); }, 300);
}

// ---------------------------------------------------------------- run
function execute(trigger = 'auto') {
  if (!ready) {
    dbg('run-skipped', `wasm not ready (trigger: ${trigger})`);
    if (trigger === 'button' || trigger === 'keyboard') els.status.textContent = 'WebAssembly is still loading or failed to load - see the badge at the top.';
    return;
  }
  const options = collectOptions();
  dbg('run-start', { trigger, operation: els.op.value, options, sourceChars: els.source.value.length });
  const t0 = performance.now();
  let result;
  let raw = '';
  try {
    raw = run(els.op.value, els.source.value, JSON.stringify(options));
    result = JSON.parse(raw);
  } catch (e) {
    dbg('run-exception', { message: String(e), stack: e && e.stack ? String(e.stack).split('\n').slice(0, 4).join(' | ') : null, rawChars: raw.length });
    result = { ok: false, operation: els.op.value, output: '', kind: 'text', error: { stage: 'wasm', message: String(e), line: null, column: null }, diagnostics: [], stats: {}, tokens: null };
  }
  const ms = performance.now() - t0;
  dbg('run-end', { ok: result.ok, ms: Number(ms.toFixed(2)), rawChars: raw.length, kind: result.kind, error: result.error || null, diagnostics: (result.diagnostics || []).length, stats: result.stats || {} });
  lastResult = result;
  renderError(result);
  renderDiagnostics(result.diagnostics || []);
  renderOutput();
  renderStatus(result, ms);
}

function renderError(r) {
  const err = r.error;
  errorLine = err && err.line ? err.line : null;
  renderGutter();
  if (!err) { els.errorBox.classList.add('hidden'); return; }
  els.errorBox.classList.remove('hidden');
  els.errorStage.textContent = err.stage;
  els.errorPos.textContent = err.line ? ` at line ${err.line}${err.column ? ', column ' + err.column : ''}` : '';
  els.errorMsg.textContent = err.message;
  const canGoto = !!err.line && !String(err.stage).startsWith('round-trip');
  els.errorGoto.classList.toggle('hidden', !canGoto);
  els.errorGoto.onclick = canGoto ? () => gotoLine(err.line, err.column) : null;
}

function renderDiagnostics(diags) {
  els.diagBox.classList.toggle('hidden', diags.length === 0);
  els.diagList.replaceChildren(...diags.map((d) => {
    const li = document.createElement('li');
    li.textContent = `${d.line}:${d.column} ${d.severity} [${d.code}] ${d.message}`;
    return li;
  }));
}

function renderStatus(r, ms) {
  const parts = [`${r.operation}: ${r.ok ? 'ok' : 'failed'}`];
  const s = r.stats || {};
  if (s.program) parts.push(`program ${s.program}`);
  if (s.tokens != null) parts.push(`${s.tokens} tokens`);
  if (s.format) parts.push(`format ${s.format}`);
  if (s.round_trip != null) parts.push(`round trip ${s.round_trip ? 'PASSED' : 'FAILED'}`);
  parts.push(`${ms.toFixed(1)} ms`);
  els.status.textContent = parts.join(' | ');
}

// ---------------------------------------------------------------- output views
function renderOutput() {
  els.tabs.forEach((t) => {
    const on = t.dataset.view === currentView;
    t.classList.remove(...TAB_ON, ...TAB_OFF);
    t.classList.add(...(on ? TAB_ON : TAB_OFF));
    t.setAttribute('aria-selected', String(on));
  });
  const r = lastResult;
  if (!r) return;
  if (!r.ok && currentView !== 'json') {
    els.output.replaceChildren(placeholder('No output: the transformer reported an error (see above).'));
    return;
  }
  switch (currentView) {
    case 'tree': els.output.replaceChildren(renderTree(debugToTree(r.output))); break;
    case 'table': els.output.replaceChildren(renderTokenTable(r.tokens || [])); break;
    case 'json': els.output.replaceChildren(pre(JSON.stringify(r, null, 2))); break;
    default: els.output.replaceChildren(pre(r.output || '(empty output)'));
  }
}

function pre(text) {
  const p = document.createElement('pre');
  p.className = 'm-0 whitespace-pre';
  p.textContent = text;
  return p;
}

function placeholder(text) {
  const p = document.createElement('p');
  p.className = 'font-sans text-sm text-slate-500 dark:text-slate-400';
  p.textContent = text;
  return p;
}

// Turn Rust `{:#?}` pretty-debug output into a nested tree using its
// bracket structure. Purely presentational.
function debugToTree(text) {
  const root = { label: '', children: [] };
  const stack = [root];
  for (const raw of text.split('\n')) {
    const line = raw.trim();
    if (!line) continue;
    if (/^[\]})],?$/.test(line)) { if (stack.length > 1) stack.pop(); continue; }
    const opens = /[{[(]$/.test(line);
    const node = { label: opens ? line.slice(0, -1).trim() : line, children: [], opens };
    stack[stack.length - 1].children.push(node);
    if (opens) stack.push(node);
  }
  return root.children;
}

function labelEl(text) {
  const span = document.createElement('span');
  const m = /^([A-Za-z_][\w]*):\s?(.*)$/.exec(text);
  if (m) {
    const k = document.createElement('span');
    k.className = 'text-sky-700 dark:text-sky-300';
    k.textContent = m[1];
    span.append(k, document.createTextNode(': '));
    text = m[2];
  }
  const v = document.createElement('span');
  if (/^"/.test(text)) v.className = 'text-emerald-700 dark:text-emerald-300';
  else if (/^-?\d/.test(text)) v.className = 'text-amber-700 dark:text-amber-300';
  else if (/^(None|true|false),?$/.test(text)) v.className = 'text-fuchsia-700 dark:text-fuchsia-300';
  else if (/^[A-Z]/.test(text)) v.className = 'font-semibold';
  v.textContent = text;
  span.appendChild(v);
  return span;
}

function renderTree(nodes, depth = 0) {
  const ul = document.createElement('ul');
  ul.className = depth === 0 ? 'tree m-0 list-none p-0' : 'm-0 list-none border-l border-slate-200 pl-4 dark:border-slate-700';
  for (const n of nodes) {
    const li = document.createElement('li');
    if (n.opens) {
      const d = document.createElement('details');
      d.open = depth < 3;
      const s = document.createElement('summary');
      s.className = 'rounded hover:bg-slate-100 dark:hover:bg-slate-800';
      s.appendChild(labelEl(n.label || '(item)'));
      if (!d.open && n.children.length) {
        const c = document.createElement('span');
        c.className = 'ml-2 text-xs text-slate-400';
        c.textContent = `${n.children.length}`;
        s.appendChild(c);
      }
      d.append(s, renderTree(n.children, depth + 1));
      li.appendChild(d);
    } else {
      li.className = 'pl-4';
      li.appendChild(labelEl(n.label));
    }
    ul.appendChild(li);
  }
  if (depth === 0 && nodes.length === 0) return placeholder('(empty)');
  return ul;
}

function renderTokenTable(tokens) {
  const table = document.createElement('table');
  table.className = 'w-full border-collapse text-left';
  const head = document.createElement('thead');
  head.innerHTML = '<tr class="text-xs text-slate-500 dark:text-slate-400"><th class="py-1 pr-3 font-medium">#</th><th class="py-1 pr-3 font-medium">Ln:Col</th><th class="py-1 pr-3 font-medium">Kind</th><th class="py-1 font-medium">Lexeme</th></tr>';
  const body = document.createElement('tbody');
  tokens.forEach((t, i) => {
    const tr = document.createElement('tr');
    tr.className = 'cursor-pointer border-t border-slate-100 hover:bg-slate-100 dark:border-slate-800 dark:hover:bg-slate-800';
    tr.title = 'Go to token';
    for (const [text, cls] of [[String(i), 'text-slate-400'], [`${t.line}:${t.column}`, 'text-slate-500 dark:text-slate-400'], [t.kind, 'text-sky-700 dark:text-sky-300'], [t.lexeme, '']]) {
      const td = document.createElement('td');
      td.className = `py-0.5 pr-3 align-top ${cls}`;
      td.textContent = text;
      tr.appendChild(td);
    }
    tr.addEventListener('click', () => gotoLine(t.line, t.column));
    body.appendChild(tr);
  });
  table.append(head, body);
  return table;
}

els.expandAll.addEventListener('click', () => els.output.querySelectorAll('details').forEach((d) => { d.open = true; }));
els.collapseAll.addEventListener('click', () => els.output.querySelectorAll('details').forEach((d) => { d.open = false; }));
els.copy.addEventListener('click', async () => {
  if (!lastResult) return;
  const text = currentView === 'json' ? JSON.stringify(lastResult, null, 2) : lastResult.output;
  try { await navigator.clipboard.writeText(text); els.copy.textContent = 'Copied'; }
  catch (e) { els.copy.textContent = 'Copy failed'; }
  setTimeout(() => { els.copy.textContent = 'Copy'; }, 1200);
});

// ---------------------------------------------------------------- boot
syncOptions();
renderGutter();
els.wasmStatus.classList.remove('hidden');
dbg('boot', 'loading WebAssembly module');
try {
  const tInit = performance.now();
  await init();
  ready = true;
  dbg('wasm-ready', `${(performance.now() - tInit).toFixed(1)} ms`);
  const v = JSON.parse(version());
  els.wasmStatus.textContent = `wasm ready, wrapper v${v.wrapper}`;
  els.wasmStatus.className = 'hidden rounded-full bg-emerald-100 px-2 py-0.5 text-xs font-medium text-emerald-800 sm:inline dark:bg-emerald-900/40 dark:text-emerald-300';
  els.run.disabled = false;
} catch (e) {
  dbg('wasm-failed', String(e));
  els.wasmStatus.textContent = 'wasm failed to load';
  els.wasmStatus.className = 'rounded-full bg-rose-100 px-2 py-0.5 text-xs font-medium text-rose-800 dark:bg-rose-900/40 dark:text-rose-300';
  els.output.replaceChildren(placeholder(`Could not load the WebAssembly module: ${e}`));
}
await loadSamples();
