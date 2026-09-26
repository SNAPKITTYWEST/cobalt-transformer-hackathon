# COBOL Transformer Attention Kernel

Complete, self-contained, executable native COBOL implementation of the scaled dot-product attention kernel.

```
SCORE(I,J) = ( Σ_K Q-VAL(I,K) × K-VAL(J,K) ) / 8
```

Sequence length = 64, head dimension = 64, √dₖ = 8.

The `CALCULATE-ATTENTION` paragraph in `src/attention.cob` is the golden reference. No pseudocode, no external BLAS/CUDA/Python call replaces the computation.

---

## Source tree

```
cobol-attention/
├── src/
│ └── attention.cob # production kernel + full pipeline
├── tests/
│ └── attention-test.cob # deterministic validation suite
├── data/
│ └── test-vectors.dat # optional external tensor data
└── README.md
```

---

## Build & run (GnuCOBOL)

```bash
# Production kernel
cobc -x -free -O2 src/attention.cob -o attention
./attention

# Validation suite
cobc -x -free -O2 tests/attention-test.cob -o attention-test
./attention-test
```

IBM Enterprise COBOL / Micro Focus equivalents use the same sources (minor adjustments only for `FUNCTION EXP` availability or floating-point representation).

---

## Kernel (exact required form)

```cobol
CALCULATE-ATTENTION.
    PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
        PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
            MOVE 0 TO SCORE(I, J)

            PERFORM VARYING K FROM 1 BY 1 UNTIL K > 64
                COMPUTE SCORE(I, J) =
                    SCORE(I, J) +
                    (Q-VAL(I, K) * K-VAL(J, K))
            END-PERFORM

            COMPUTE SCORE(I, J) =
                SCORE(I, J) / 8
        END-PERFORM
    END-PERFORM.
```

Transpose semantics are intentional: `Kᵀ(K,J) = K(J,K)`.

---

## Full pipeline

| Stage | Procedure |
|--------------------------|------------------------|
| Q × Kᵀ / √dₖ | `CALCULATE-ATTENTION` |
| Numerically stable softmax | `SOFTMAX-ATTENTION` |
| Softmax × V | `APPLY-VALUE-MATRIX` |

All stages are pure COBOL loops. No external matrix library is called.

---

## Initialization modes

Controlled by `WS-TEST-MODE`:

| Mode | Meaning |
|------|---------|
| `Z` | Zero matrices |
| `I` | Identity-compatible (default) |
| `C` | Constant (all ones) |
| `M` | Mixed positive/negative deterministic pattern |
| `E` | Load from `data/test-vectors.dat` (fallback to identity) |

---

## Validation

The test suite verifies every one of the 4 096 elements for:

- Zero → output ≈ 0
- Identity → SCORE(I,J) = δ(I,J)/8
- Constant → SCORE = 8 everywhere
- Mixed → independent recomputation of the triple product matches

Reports: mismatch count, maximum absolute error, PASS/FAIL, non-zero `RETURN-CODE` on failure.

---

## Hot-loop profile

```
I × J × K = 64 × 64 × 64 = 262 144 multiply-accumulates
```

Opportunities (correctness preserved):

- Contiguous row-major storage (already present)
- Base-address hoisting of the current Q-row / K-row
- Register accumulator instead of repeated table store/load
- Loop tiling for cache
- Compiler `OPT(2)` / `-O2`
- Optional `CALL "ATTN-GEMM"` backend (HLASM / other) that must implement the identical mathematical contract

---

## Optional accelerator interface

```cobol
CALL "ATTN-GEMM" USING Q-TABLE K-TABLE SCORE-TABLE
    ON EXCEPTION
        *> fall back to native COBOL reference
END-CALL
```

The native COBOL implementation remains the executable reference.
cobc -x -free -O2 src/attention.cob -o attention
./attention

cobc -x -free -O2 tests/attention-test.cob -o attention-test
./attention-test
