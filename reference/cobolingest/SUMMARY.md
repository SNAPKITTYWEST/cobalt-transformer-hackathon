# cobolingest — reference notes

A third-party COBOL-estate ingestion tool, shared for comparison while
deciding what belongs in this submission. **The source code was not
provided** — only the description below, from the author's own summary.
Nothing here has been verified by this repo; treat every claim as reported,
not confirmed.

## What it claims to do

A Python tool (standard library only, Python 3.9+, git optional) with a CLI
and test suite, for ingesting a raw mainframe COBOL estate:

- **Content-based file classification** — sorts files into programs,
  copybooks, DCLGENs, JCL, PROCs, BMS maps, and docs by looking at file
  content, not extension (so extensionless or `.txt` mainframe exports are
  still recognized).
- **EBCDIC handling** — reads EBCDIC source, including raw FB80 (80-byte,
  no line breaks) binary FTP downloads; keeps the original bytes and stores
  a UTF-8 copy alongside for reading. Default code page `cp037` (US);
  European shops may need `cp1140` or `cp273`.
- **COBOL parsing** — extracts COPY, static/dynamic CALL, embedded SQL
  (tables read/written), CICS commands (LINK, XCTL, MAP, FILE),
  `SELECT ... ASSIGN` (file-to-DD-name link), paragraphs, code metrics.
  Skips columns 1–6 and 73–80, comments, debug lines, and literals.
- **JCL parsing** — steps, `EXEC PGM`/`PROC`, DDs with DISP, continuation
  lines, concatenations; also finds programs in in-stream data (e.g.
  `RUN PROGRAM(x)` under IKJEFT01, IMS `DFSRRC00` PARM strings).
- **Reference resolution** — every reference is tagged exactly one of:
  resolved, ambiguous, system, missing, dynamic, external. Unknowns are
  listed in a `GAPS.md` file with location, not silently guessed.
- **Batch data lineage** — job → step → program (incl. CALLed programs) → DD
  → dataset.
- **Impact analysis** — e.g. "what runs if I change CUSTREC" across the
  whole estate.
- **Integrity** — every copied file is re-hashed; a `verify` command re-hashes
  the whole repo; reruns on unchanged input are byte-identical, so git only
  records real changes.

## Evidence offered (as reported, not independently checked)

- 20 tests passing, including an end-to-end run on a sample estate with an
  EBCDIC copybook, an extensionless member, JCL, and docs.
- **Mutation testing on the parser's own guarantees**: the author
  deliberately broke 6 invariants one at a time and checked the test suite
  caught each one. On the first pass, 2 of 6 were *not* caught — the
  fixtures were too weak — and were then strengthened until all 6 were
  caught. This is the single most convincing piece of evidence in the
  writeup, because passing tests alone don't prove a suite is strict; only
  showing a broken invariant gets caught does.

## Stated caveats (from the author)

- Tested on Linux only; the Windows-specific code (drive letters, junction
  skipping) is untested — start with `--dry-run` on Windows.
- SQL table extraction and lineage are pattern-based, not verified against
  a real SQL parser.
- JCL symbolic parameters (`&HLQ`) are not substituted and PROCs are not
  expanded, which limits lineage for jobs that call PROCs.
- The system-name lists only cover IBM names the author is confident about;
  shop-specific vendor utilities will show up as "missing" until added.

## Why this is here

Kept for comparison against this repo's own approach while deciding what to
submit. The mutation-testing technique — break a guarantee on purpose,
confirm the test suite catches it, tighten the tests if it doesn't — is
worth applying to `cobol-transformer`'s own parser/CFG before final
submission; see [SUBMISSION.md](../../SUBMISSION.md).
