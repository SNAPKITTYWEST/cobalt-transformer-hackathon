// Assemble web/dist from web/src, the wasm-pack output, and sample COBOL
// files copied verbatim from elsewhere in this repo. Run after build:wasm;
// build:css writes dist/app.css afterwards.
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync, statSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const repo = resolve(web, '..');
const dist = join(web, 'dist');
const pkg = join(web, 'wasm', 'pkg');

// Real files from the repo. `note` is shown under the picker.
export const SAMPLES = [
  { id: 'hello', path: 'cobol-transformer/tests/fixtures/hello.cob',
    note: "the transformer's own integration-test fixture" },
  { id: 'ledger-post', path: 'sources/C/Users/jessi/GolandProjects/devflow-finance-twin/finance/cobol/LEDGER_POST.cbl',
    note: 'indexed ledger posting program' },
  { id: 'law-kernel', path: 'sources/D/tmp/cartographer-agent/kernels/cobol-law-kernel.cob',
    note: 'claim-routing agent kernel' },
  { id: 'reqparse', path: 'cobalt/REQPARSE.cbl',
    note: 'Cobalt HTTP request-line nugget; the lexer currently stops on a non-ASCII character in a comment' },
  { id: 'vault-treasury', path: 'sources/C/Users/jessi/Desktop/SKC/DEVFLOW-FINANCE/bridges/cobol/vault_treasury.cbl',
    note: 'uses operators the lexer does not accept yet' },
  { id: 'mamari', path: 'sources/C/Users/jessi/Desktop/the-49th-call/substrate/mamari.cbl',
    note: 'uses operators the lexer does not accept yet' },
  { id: 'vector-add', path: 'gpu-cobol/examples/vector-add.cbl',
    note: 'GPU-COBOL dialect example, shown as source; the transformer does not support this dialect' },
];

if (!existsSync(join(pkg, 'cobol_transformer_wasm_bg.wasm'))) {
  console.error('web/wasm/pkg is missing - run `npm run build:wasm` first.');
  process.exit(1);
}

rmSync(dist, { recursive: true, force: true });
mkdirSync(join(dist, 'pkg'), { recursive: true });
mkdirSync(join(dist, 'samples'), { recursive: true });

cpSync(join(web, 'src', 'index.html'), join(dist, 'index.html'));
cpSync(join(web, 'src', 'app.js'), join(dist, 'app.js'));
for (const f of ['cobol_transformer_wasm.js', 'cobol_transformer_wasm_bg.wasm']) {
  cpSync(join(pkg, f), join(dist, 'pkg', f));
}

const manifest = [];
for (const s of SAMPLES) {
  const src = join(repo, s.path);
  const ext = s.path.slice(s.path.lastIndexOf('.'));
  const file = `${s.id}${ext}`;
  cpSync(src, join(dist, 'samples', file));
  manifest.push({ id: s.id, label: `${s.path.split('/').pop()} (${s.path.split('/')[0]})`, path: s.path, file, note: s.note, bytes: statSync(src).size });
}
writeFileSync(join(dist, 'samples', 'samples.json'), JSON.stringify(manifest, null, 2) + '\n');
// GitHub Pages: serve files as-is (no Jekyll processing).
writeFileSync(join(dist, '.nojekyll'), '');

console.log(`dist assembled: ${manifest.length} samples, wasm ${statSync(join(dist, 'pkg', 'cobol_transformer_wasm_bg.wasm')).size} bytes`);
