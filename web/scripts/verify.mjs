// End-to-end check of the built site:
//  1. load dist/pkg (the exact files the browser gets) in Node and call every
//     exported wasm function on cobol-transformer/tests/fixtures/hello.cob;
//  2. serve dist under a sub-path and fetch every asset the page needs.
// Exits non-zero on any failure.
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { startServer } from './serve.mjs';

const web = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const repo = resolve(web, '..');
const dist = join(web, 'dist');
let failures = 0;
const check = (cond, msg) => { console.log(`${cond ? 'PASS' : 'FAIL'}  ${msg}`); if (!cond) failures++; };

// ---- 1. wasm exports
const wasm = await import(pathToFileURL(join(dist, 'pkg', 'cobol_transformer_wasm.js')).href);
wasm.initSync({ module: readFileSync(join(dist, 'pkg', 'cobol_transformer_wasm_bg.wasm')) });
const hello = readFileSync(join(repo, 'cobol-transformer/tests/fixtures/hello.cob'), 'utf8');
const exported = Object.keys(wasm).filter((k) => typeof wasm[k] === 'function' && !['default', 'initSync'].includes(k)).sort();
console.log(`wasm exports: ${exported.join(', ')}`);

const v = JSON.parse(wasm.version());
check(v.operations.length === 9, `version() -> ${JSON.stringify(v)}`);

const calls = {
  parse: () => wasm.parse(hello),
  tokens: () => wasm.tokens(hello),
  dump_ast: () => wasm.dump_ast(hello),
  dump_symbols: () => wasm.dump_symbols(hello),
  dump_cfg: () => wasm.dump_cfg(hello),
  generate: () => wasm.generate(hello, 'normalize,modernize'),
  round_trip: () => wasm.round_trip(hello),
  detect_format: () => wasm.detect_format(hello),
  normalize_format: () => wasm.normalize_format(hello, 'auto'),
};
for (const [name, fn] of Object.entries(calls)) {
  const r = JSON.parse(fn());
  const first = r.output.split('\n').filter(Boolean).slice(0, 2).join(' / ');
  check(r.ok && r.output.length > 0, `${name}(hello.cob): ok=${r.ok} ${JSON.stringify(r.stats)} :: ${first.slice(0, 90)}`);
}
const gen = JSON.parse(wasm.generate(hello, ''));
check(gen.output.includes('PROGRAM-ID. HELLO.') && gen.output.includes('DISPLAY GREETING.'), 'generate() output contains PROGRAM-ID. HELLO. and DISPLAY GREETING.');
const rt = JSON.parse(wasm.round_trip(hello));
check(rt.stats.round_trip === true, 'round_trip() reports PASSED');
const sym = JSON.parse(wasm.dump_symbols(hello));
check(sym.output.includes('"GREETING"') && sym.output.includes('"MAIN-PARA"'), 'dump_symbols() lists GREETING and MAIN-PARA');

// run() dispatcher + error positions
const r1 = JSON.parse(wasm.run('ast', hello, JSON.stringify({ file_name: 'hello.cob' })));
check(r1.ok && r1.stats.program === 'HELLO', `run("ast") -> program ${r1.stats.program}, ${r1.stats.tokens} tokens`);
const badLex = JSON.parse(wasm.run('parse', hello.replace('DISPLAY GREETING.', 'DISPLAY GREETING = 1.'), '{}'));
check(!badLex.ok && badLex.error.stage === 'lex' && badLex.error.line === 10, `lexer error surfaced with position: ${JSON.stringify(badLex.error)}`);
const badParse = JSON.parse(wasm.run('parse', hello.replace('PROGRAM-ID. HELLO.', 'PROGRAM-ID. .'), '{}'));
check(!badParse.ok && badParse.error.stage === 'parse' && badParse.error.line === 2, `parser error surfaced with position: ${JSON.stringify(badParse.error)}`);
const badOpt = JSON.parse(wasm.run('nope', hello, '{}'));
check(!badOpt.ok && badOpt.error.stage === 'options', `unknown operation rejected: ${badOpt.error.message}`);

// ---- 2. static serving under a sub-path
const base = '/cobalt-transformer-hackathon/';
const server = await startServer(0, base);
const origin = `http://127.0.0.1:${server.address().port}`;
const get = async (p) => { const res = await fetch(origin + base + p); return { res, body: Buffer.from(await res.arrayBuffer()) }; };
try {
  const { res, body } = await get('');
  const html = body.toString('utf8');
  check(res.status === 200 && html.includes('<title>COBOL Transformer Playground</title>'), `GET ${base} -> ${res.status}, ${body.length} bytes, title found`);
  const refs = [...html.matchAll(/(?:href|src)="([^"]+)"/g)].map((m) => m[1]);
  check(refs.every((u) => u.startsWith('./')), `all asset references are relative: ${refs.join(', ')}`);
  for (const p of ['app.css', 'app.js', 'pkg/cobol_transformer_wasm.js', 'pkg/cobol_transformer_wasm_bg.wasm', 'samples/samples.json']) {
    const r = await get(p);
    check(r.res.status === 200 && r.body.length > 0, `GET ${base}${p} -> ${r.res.status} ${r.res.headers.get('content-type')} ${r.body.length} bytes`);
  }
  const css = (await get('app.css')).body.toString('utf8');
  check(css.includes('.dark\\:bg-slate-950'), 'app.css contains compiled Tailwind dark-mode utilities');
  const manifest = JSON.parse((await get('samples/samples.json')).body.toString('utf8'));
  for (const s of manifest) {
    const r = await get(`samples/${s.file}`);
    const same = r.body.equals(readFileSync(join(repo, s.path)));
    const res = JSON.parse(wasm.run('parse', r.body.toString('utf8'), JSON.stringify({ file_name: s.path })));
    const outcome = res.ok ? `parses (${res.stats.program})` : `${res.error.stage} error at ${res.error.line}:${res.error.column}: ${res.error.message}`;
    check(r.res.status === 200 && same, `sample ${s.id}: served byte-identical to ${s.path}; ${outcome}`);
  }
  const wasmMod = await WebAssembly.compile((await get('pkg/cobol_transformer_wasm_bg.wasm')).body);
  check(WebAssembly.Module.exports(wasmMod).length > 0, `served .wasm compiles (${WebAssembly.Module.exports(wasmMod).length} module exports)`);
} finally {
  await new Promise(resolve => server.close(resolve));
}

console.log(failures ? `\n${failures} check(s) FAILED` : '\nall checks passed');
process.exitCode = failures ? 1 : 0;
