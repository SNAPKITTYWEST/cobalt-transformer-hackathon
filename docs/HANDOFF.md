# Handoff — Cobalt COBOL transformer (hackathon + commercial)

State as of 2026-09-26. Everything below was checked against the repos when
this was written. Re-check before acting; IBM Bob and other agents also edit
these folders.

## 1. The three repos

| Local path | GitHub (private, SNAPKITTYWEST) | HEAD | State |
|---|---|---|---|
| `C:\Users\jessi\Desktop\cobol-transformer-repo` | none (origin repo) | `c6ae1f1` | **Uncommitted:** `cobalt/attention-drafts/05, 06, 10` (completed by an agent) and `docs/agent-runs/LOG.md` (untracked) |
| `C:\Users\jessi\Desktop\cobalt-transformer` | `SNAPKITTYWEST/cobalt-transformer` (commercial) | `c6ae1f1` | Clean. **Never pushed**; the remote is empty |
| `C:\Users\jessi\Desktop\cobalt-transformer-hackathon` | `SNAPKITTYWEST/cobalt-transformer-hackathon` | `046c2c6` | Clean. **Never pushed**; the remote is empty |

Both split repos were cloned from `cobol-transformer-repo` at `c6ae1f1` and share
the same core content:

- `cobol-transformer/`: Bob's Rust COBOL transformer.
- `cobalt/`: the Haskell compiler, plus REQPARSE, the bank suite and the attention files.
- `gpu-cobol/`: the C GPU-COBOL compiler.
- `sources/`: collected COBOL.
- `docs/`

The hackathon repo additionally has `web/` (the playground), `.github/workflows/pages.yml`,
`docs/HACKATHON.md` and this file.

## 2. What was done

| Commit | Repo | What |
|---|---|---|
| `c6ae1f1` | all three | GPU-COBOL compiler completed: lexer, parser, AST, IR and common. Builds with `gcc -std=c11 -Wall -Wextra -O2 -Iinclude -o gpu-cobol.exe src/*.c` (MinGW gcc at `C:\Strawberry\c\bin`). `--emit-ast/--emit-ir/--emit-ptx examples/vector-add.cbl` all exit 0 |
| `362f09c` | hackathon | Web playground: the Rust `cobol-transformer` crate compiled to wasm32 through `web/wasm` (wasm-bindgen), with a Tailwind 4 static UI in `web/dist` (committed, ~560 KB) |
| `4eba173` | hackathon | Debug mode: a "debug" checkbox or `?debug=1` writes an on-page log and console output for every run |
| `c54f466` | hackathon | Ctrl+Enter fixes: ignore auto-repeat and IME Enter; cancel the pending run-on-edit timer |
| `046c2c6` | hackathon | New Execute path. The button is a form submit. Runs requested before wasm loads are queued and run once the sample is loaded. Adds a busy state, run history with restore, and an output flash. It uses `setTimeout`, not `requestAnimationFrame`, because rAF is paused in hidden frames |

### Web playground: build and verify
```
cd web
rustup target add wasm32-unknown-unknown   # once
npm ci
npm run build      # wasm-pack → build-site → tailwind
npm run verify     # must end with "all checks passed"
python -m http.server 8080 -d dist        # then open http://localhost:8080/?debug=1
```
Changes under `web/src` alone need only `npm run build:site && npm run build:css`,
because the wasm in `web/wasm/pkg` is reused.

In-browser testing: headless Chrome (`--headless=new --virtual-time-budget=20000
--dump-dom`) loaded a throwaway harness page in `dist/` that drove the playground in
an iframe. Delete any harness file before committing.

## 3. Open work, in priority order

1. **The user says the Run/Execute button still fails for them. This is unconfirmed after `046c2c6`.** Ask them for the `?debug=1` log after pressing Execute; it shows exactly where it stops (`execute-submit` → `run-start` → `run-end`). In headless Chrome every path works.
2. **Transformer lexer gaps (`cobol-transformer/src/lexer.rs`).** 4 of 7 playground samples stop in the lexer:
   - `=` (mamari, vector-add)
   - `<` (vault_treasury)
   - an em-dash in a comment (REQPARSE)

   Fix these in the crate, not the web layer, then rebuild the wasm. 3 crate unit tests already fail: `test_lex_level_number`, `test_lex_keywords`, `test_replace_statement`.
3. **GPU-COBOL `src/ptx.c` emits PTX that would not run on a GPU:**
   - integer offset math is emitted as `.f32`;
   - parameters are never loaded (`ld.param`);
   - load and store ignore the offset and the mask;
   - float constants are written as `0f00000000`.

   The IR is correct; only the emitter needs fixing.
4. **Attention drafts** (in `cobol-transformer-repo` only, uncommitted):
   - 05, 06 and 10 are finished.
   - 01, 02, 07 and 09 still contain a literal `...` line.
   - The user **rejected** the agent's edit to 07. Ask whether to finish them, commit what's done, or leave them.
   - Pasted source problems, not yet changed: `01 ZERO USAGE COMP-1` in 05, 06 and 07 (ZERO is reserved); file 01 gives its tables and their elements the same names.
5. **Builder agent: not started.** Build the vertical slice from `docs/MASTER-PROMPT-byox-mxml-nuggets.md` §27 in a `training/` folder: BYOX corpus → MXML nuggets → XSD → XMLDSig (SHA-256, tamper rejection) → Mustache → sandbox → fail/repair/pass → signed result and trajectory. `cobalt/REQPARSE.cbl` (HTTP request-line, nuggets N01–N07) is the existing first-slice idea. `cobc` is **not** installed, so the executable slice must use Python, C, Rust or Go.
6. **Reward agent: not started.** After the above, score every agent run from `cobol-transformer-repo/docs/agent-runs/LOG.md` against its real commits and outputs, and write `REWARD.md` next to the log.
7. **Sync the split repos.** The commercial repo does not have the web commits, which is intended. Neither split repo has the agent log or the attention-draft completions.

## 4. Pushing and hosting

- The user's global pre-push hook **blocks agent pushes** (`C:\Users\jessi\.git-hooks\pre-push`). Give the user commands instead:
  ```powershell
  cd C:\Users\jessi\Desktop\cobalt-transformer; git push -u origin main
  cd C:\Users\jessi\Desktop\cobalt-transformer-hackathon; git push -u origin main
  ```
  They type `y` at the "Do you want to push?" prompt.
- The hackathon repo was made **public** (user's decision, 2026-09-26) so GitHub Pages can publish on the Free plan. Pages source is "GitHub Actions"; `.github/workflows/pages.yml` deploys on pushes to `main`. The commercial repo stays **private**.

## 5. Working rules for this user

- **No `Co-Authored-By` lines** in commits. A global commit-msg hook also strips them.
- **When the user pastes code, save it to the repo as given** and commit. Don't search the disk or send agents looking for "originals". Pastes over 50,000 characters arrive truncated; say exactly where one stopped and ask for the rest.
- **Don't add "untested" caveats** to code the user or Bob has already run.
- **Don't fabricate or rewrite Bob's work** (IBM Bob edits concurrently). Locate existing files first.
- **If a permission prompt rejects a tool call, stop and report. Never retry it.** In an earlier run, one agent retried a rejected write; don't repeat that.
- Stage specific paths, never `git add -A` blindly. `sovereign-engine-v2` has 1,811 `target/` build files tracked by git by mistake; don't make it worse.
- Be brief. Verify claims yourself (rebuild, re-run) before reporting them.
- Record every agent run in `cobol-transformer-repo/docs/agent-runs/LOG.md`.

## 6. Machine

AMD Ryzen 7 7700X (reporting 8 threads, so SMT looks disabled), 31 GB RAM, RTX 3080,
2 × 2 TB NVMe. Toolchains: Rust + wasm-pack, Node 25, Python 3.12, MinGW gcc 13.2
(Strawberry). Not installed: GnuCOBOL (`cobc`) and `make` (use `C:\Strawberry\c\bin\gmake`).
