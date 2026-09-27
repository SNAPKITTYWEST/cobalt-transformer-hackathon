# Submission: Cobalt COBOL Transformer

**Live demo:** https://snapkittywest.github.io/cobalt-transformer-hackathon/
**Repo:** https://github.com/SNAPKITTYWEST/cobalt-transformer-hackathon (public)
**Target:** lablab.ai IBM AI Challenge 2026

This is the single entry point for everything in this submission — what's
here, what actually works today, and where to read more. Every claim below is
checked against a build or test run; where something is estimated or
unfinished, it's labeled as such rather than presented as done.

## What this is

A COBOL transformation toolchain built around one idea: **ground the AI on
real parser output, not raw text.** IBM's own field research
(`docs/IBM-HACKATHON-FIELD-GUIDE.md`) states there is no public,
hackathon-ready COBOL parsing/transformation API — teams are expected to send
raw source into a general-purpose model. We instead run a real hand-written
COBOL parser first (paragraphs, data items, symbol table, control-flow
graph), and ground Granite's prompts on that structure.

Four independent, working pieces:

| Piece | What it is | Status |
|---|---|---|
| [`cobol-transformer/`](cobol-transformer/) | A real Rust COBOL parser: lexer, parser, AST, symbol table, type system, CFG, dataflow, codegen | Builds clean, unit tests pass (some pre-existing lexer gaps — see below) |
| [Web playground](docs/HACKATHON.md) (`web/`) | That parser compiled to WebAssembly, running live in the browser via a Tailwind UI — **no server, no API key, nothing leaves the visitor's machine** | **Live and verified**, see below |
| [IBM watsonx backend](docs/IBM-BACKEND.md) (`cobol-transformer/src/ibm/`, `service/`) | Two implementations grounding Granite (explain/translate/test-gen) on real parser facts, plus Object Storage and Code Engine deploy helpers | Rust side builds and tests pass (11/11, network-gated tests correctly `ignored`); Python service written, not yet run — both need IBM credentials this machine doesn't have |
| [`gpu-cobol/`](docs/GPU-COBOL.md) | A second, independent compiler: a COBOL-syntax dialect for GPU kernels, emitting PTX | Lexer → parser → AST → IR all correct and tested; the final PTX-emission stage has known bugs, documented, not hidden |

Plus: [`cobalt/`](cobalt/) (a hand-rolled Haskell COBOL compiler and an
attention-kernel implementation) and [`sources/`](sources/) (37 real COBOL
programs collected as a test corpus, cataloged in
[docs/COBOL-PROGRAMS.md](docs/COBOL-PROGRAMS.md)).

## Try it right now

No install, no credentials:

**https://snapkittywest.github.io/cobalt-transformer-hackathon/**

Pick a sample or paste COBOL, choose an operation (parse, AST, symbols, CFG,
generate, round-trip, format detect/normalize), see real output from the real
parser running in your browser. `?debug=1` shows a live log of every
execution. Details: [docs/HACKATHON.md](docs/HACKATHON.md).

## What's honestly incomplete

Stated up front, not discovered by a judge later:

1. **Parser lexer gaps.** 4 of 7 samples in the playground stop at the
   lexer on `=`, `<`, or a non-ASCII character in a comment (see
   [docs/HANDOFF.md](docs/HANDOFF.md) §3.2). These are real COBOL programs
   collected from other projects, kept in the demo specifically so the gap is
   visible rather than cherry-picked away.
2. **GPU-COBOL's PTX emitter** produces syntactically valid but not
   execution-correct PTX (integer math emitted as float ops, parameters never
   loaded, masks ignored). Full detail: [docs/GPU-COBOL.md](docs/GPU-COBOL.md).
3. **The IBM backend needs live credentials** neither implementation has on
   this machine. What's verified without credentials, and what isn't, is in
   [docs/IBM-BACKEND.md](docs/IBM-BACKEND.md).
4. **No measured speedup numbers.** An earlier planning document for this
   project quoted fixed benchmark figures (6.2x, "4–8x") for a vectorization
   analyzer; those turned out to be hardcoded constants in the estimator, not
   measurements. This submission does not repeat them. See
   [docs/IBM-BACKEND.md](docs/IBM-BACKEND.md) → "Do not repeat unmeasured
   numbers."

## Documentation map

| Document | Covers |
|---|---|
| [README.md](README.md) | Repository layout, what's in each top-level folder |
| [docs/HACKATHON.md](docs/HACKATHON.md) | Web playground architecture, build, and how to run it |
| [docs/COBOL-PROGRAMS.md](docs/COBOL-PROGRAMS.md) | Every COBOL program in `sources/` and `cobalt/`: what it does, its I/O, its record layouts |
| [docs/GPU-COBOL.md](docs/GPU-COBOL.md) | The GPU-COBOL compiler: dialect, pipeline, build, known PTX-emitter limitation |
| [docs/IBM-BACKEND.md](docs/IBM-BACKEND.md) | Both watsonx.ai integrations (Rust and Python), endpoints, credentials, verified status |
| [docs/IBM-HACKATHON-FIELD-GUIDE.md](docs/IBM-HACKATHON-FIELD-GUIDE.md) | The IBM service research this submission's architecture is built against |
| [docs/HANDOFF.md](docs/HANDOFF.md) | Working notes for continuing this project: open tasks, repo state, verified commits |
| [docs/MASTER-PROMPT-byox-mxml-nuggets.md](docs/MASTER-PROMPT-byox-mxml-nuggets.md) | A separate, exploratory spec (COBOL-tutorial training corpus) not yet built; unrelated to the hackathon submission itself |
| [reference/README.md](reference/README.md) | Two other hackathon submissions kept for comparison, and an honest scoring of all three against each other |

## License

**Not yet added.** MIT is intended (see chat history) but the LICENSE file has
not been created — it needs a copyright holder name first. A public repo with
no LICENSE file is all-rights-reserved by default, which most hackathon rules
either disallow or can't legally use; add this before final submission. One
collected source file under `sources/` (`worm_bridge.cob`, in two locations)
carries its own AGPL-3.0-or-later header from its original author, which an
MIT LICENSE at the repo root would not override for that file — either leave
its header as an explicit exception or get the author's sign-off to relicense
it.
