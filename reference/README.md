# Reference: other hackathon submissions

Two other projects, kept here **only for comparison** while deciding what
belongs in this submission. Neither is part of the Cobalt build, neither is
built by this project, and neither should be presented as this project's own
work.

| Folder | What it is | Source |
|---|---|---|
| [`bobmigrate/`](bobmigrate/) | A Go/Fiber API translating one COBOL payroll-overtime paragraph to Go, with a Python "self-healing" test-repair loop | Full source sent by co-founder Ahmad Parr, saved as received |
| [`cobolingest/`](cobolingest/) | A Python estate-wide COBOL/JCL ingestion, parsing, and impact-analysis tool | Description only (no source shared) — see [SUMMARY.md](cobolingest/SUMMARY.md) for exactly what's known and what's just claimed |

## Honest comparison

| | bobmigrate | Cobalt (this repo) | cobolingest |
|---|---|---|---|
| Scope | One hand-picked paragraph → Go | Real lexer/parser/AST/CFG over actual COBOL | Whole-estate ingestion, parsing, lineage, impact analysis |
| Core claim actually working? | Go math: yes. Headline "AI self-repair loop": **no** — `call_llm()` is a stub returning `""` | Yes — builds, tests pass, live public demo | Reported: 20 tests passing, plus deliberate mutation testing (see below) |
| Strongest evidence given | 4 unit tests | Live demo at the public URL, `npm run verify` | Broke 6 parser invariants on purpose and confirmed the tests catch them — the single most convincing form of proof in this comparison |
| Stated honestly, not hidden | Vague TODO comment | [SUBMISSION.md](../SUBMISSION.md) states the lexer gaps and PTX-emitter bug plainly | Author lists Windows-untested code, pattern-based SQL/lineage, and unexpanded JCL PROCs |

**Takeaway carried into this submission:** cobolingest's mutation-testing
technique is worth doing to `cobol-transformer`'s own parser and CFG before
final judging — passing tests prove less than tests that were shown to catch
a deliberately introduced bug.
