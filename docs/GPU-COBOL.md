# GPU-COBOL compiler

A from-scratch C compiler (`gpu-cobol/`) for a COBOL-syntax GPU kernel dialect:
COBOL-shaped source in, PTX (NVIDIA's GPU assembly) out, by way of a real
lexer → parser → AST → IR pipeline. It is independent of `cobol-transformer/`
and `cobalt/`; it does not compile standard COBOL, only its own `KERNEL
SECTION` dialect.

## Why a COBOL-syntax GPU language

The premise: COBOL's `PARAGRAPH`/`PERFORM`/`PIC` shape can host GPU kernel
programming — grid/thread identifiers, pointer parameters, masked loads and
stores — the same primitives Triton or CUDA C expose, in COBOL's own syntax.
`examples/vector-add.cbl` is the running example:

```cobol
IDENTIFICATION DIVISION.
PROGRAM-ID. VECTOR-ADD.

DATA DIVISION.
WORKING-STORAGE SECTION.
01 N PIC 9(9) COMP-5 VALUE 1024.
01 BLOCK-SIZE PIC 9(9) COMP-5 VALUE 256.

KERNEL SECTION.
KERNEL VECTOR-ADD-KERNEL
    PARAMETER A AS GPU-POINTER FLOAT32
    PARAMETER B AS GPU-POINTER FLOAT32
    PARAMETER C AS GPU-POINTER FLOAT32
    PARAMETER N AS GPU-I32.

PROCEDURE DIVISION.
    COMPUTE PID = PROGRAM-ID-X.
    COMPUTE OFFSET = PID * BLOCK-SIZE + LANE-ID.
    SET MASK TO OFFSET < N.

    GPU-LOAD A OFFSETS OFFSET MASK MASK INTO VA.
    GPU-LOAD B OFFSETS OFFSET MASK MASK INTO VB.

    COMPUTE VC = VA + VB.

    GPU-STORE C OFFSETS OFFSET VALUE VC MASK MASK.
    GOBACK.
```

`PROGRAM-ID-X` and `LANE-ID` are the block and thread index (Triton's
`program_id`/CUDA's `threadIdx`); `GPU-LOAD`/`GPU-STORE` are masked,
offset-addressed memory ops, so `OFFSET < N` guards the last partial block the
way Triton's `mask=` argument does.

## Pipeline

```
source.cbl → lexer → parser → AST → sema (bind_globals + IR build) → IR → PTX
             (token.c) (parser.c)   (ast.c)      (sema.c)         (ir.c) (ptx.c)
```

| Stage | File(s) | Lines |
|---|---|---|
| Tokens | `include/token.h`, `src/token.c` | 225 |
| Lexer | `include/lexer.h`, `src/lexer.c` | 349 |
| Parser | `include/parser.h`, `src/parser.c` | 760 |
| AST | `include/ast.h`, `src/ast.c` | 344 |
| Semantic analysis → IR construction | `include/sema.h`, `src/sema.c` | 335 |
| IR | `include/ir.h`, `src/ir.c` | 404 |
| PTX emitter | `include/ptx.h`, `src/ptx.c` | 180 |
| Driver, arena, diagnostics | `src/main.c`, `src/common.c`, `include/common.h`, `include/gpu_cobol.h` | 571 |

**3,201 lines total**, plus `examples/vector-add.cbl` and the `Makefile`.

## Build and run

```powershell
cd gpu-cobol
gcc -std=c11 -Wall -Wextra -O2 -Iinclude -o gpu-cobol.exe src/*.c
./gpu-cobol.exe --emit-ast examples/vector-add.cbl
./gpu-cobol.exe --emit-ir  examples/vector-add.cbl
./gpu-cobol.exe --emit-ptx examples/vector-add.cbl
```
(MinGW gcc, e.g. `C:\Strawberry\c\bin\gcc.exe`, if `gcc` is not on PATH. `gmake`
also builds it via the included `Makefile`.)

All three modes exit 0 on `vector-add.cbl`. The IR is correct: it computes
`program_id * block_size + lane_id`, builds a `< N` mask, does two masked
loads, an add, and a masked store.

## Known limitation: the PTX emitter, not the front end

`src/ptx.c` emits syntactically valid PTX, but it would not execute correctly
on real hardware:

- integer offset arithmetic is emitted with `.f32` (float) instructions instead
  of `.s32`;
- kernel parameters are never loaded from their `.param` slots into registers
  before use;
- `GPU-LOAD`/`GPU-STORE` ignore the computed offset and the mask — they always
  read/write the base pointer;
- float immediates are written as the placeholder bit pattern `0f00000000`
  rather than the literal's real encoding.

The lexer, parser, AST and IR are correct and tested (see below); only the
final code-generation stage needs work before this could run on a GPU. This is
recorded here rather than glossed over, because "PTX comes out" and "PTX runs
correctly" are different claims and only the first one is true today.

## What else parses

Beyond the example, an ad-hoc program exercising more of the grammar compiled
correctly during development: lowercase keywords, `*>` comments, `**` (power),
unary minus, `IF`/`THEN`/`ELSE`/`END-IF` with `AND`/`NOT`, `GPU-DOT ... WITH
... INTO`, `GPU-REDUCE-SUM ... AXIS ... INTO`, `GPU-SYNC`, `MOVE`, and
`STOP RUN`. Also: `MASK`, `SHAPE`, `OTHER`, `AXIS`, `DATA`, and `ID` are
reserved keywords that can *also* be used as data names, which is what lets
`SET MASK TO ...` / `MASK MASK` in the example parse.

## Relationship to the rest of the submission

GPU-COBOL is a separate, standalone compiler — it shares no code with
`cobol-transformer/` (which parses real IBM/GnuCOBOL-style COBOL and does not
understand `KERNEL SECTION`) or with `cobalt/` (a different, Haskell-hosted
compiler). It is included as a second, independent demonstration that COBOL's
syntax can host a modern workload — GPU kernels — not just legacy business
logic.
