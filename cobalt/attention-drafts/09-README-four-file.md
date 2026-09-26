# COBOL Transformer Attention Kernel

Production-grade **native COBOL** implementation of the scaled dot-product attention score kernel:

```
SCORE = (Q × Kᵀ) / √dₖ
```

with

| Parameter | Value |
|------------------|-------|
| Sequence length | 64 |
| Head dimension | 64 |
| √dₖ | 8 |

The **PROCEDURE DIVISION** nested `PERFORM VARYING` loops are the golden reference. No pseudocode, no Python, no ML-framework call replaces the computation.

---

## Files

| File | Role |
|-------------------------|------|
| `attention.cob` | Self-contained program containing the full pipeline + embedded tests |
| `attention-kernel.cob` | Modular callable kernel (`ENTRY` points for each stage) |
| `attention-test.cob` | Stand-alone deterministic validation driver (4096 elements verified) |
| `attention-backend.asm` | Optional HLASM skeleton implementing the identical mathematical contract |

---

## Kernel (golden reference)

```cobol
CALCULATE-ATTENTION.
    PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
        PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
            MOVE 0 TO SCORE-ELEMENT(I, J)

            PERFORM VARYING K FROM 1 BY 1 UNTIL K > 64
                COMPUTE SCORE-ELEMENT(I, J) =
                    SCORE-ELEMENT(I, J) +
                    (Q-ELEMENT(I, K) * K-ELEMENT(J, K))
            END-PERFORM

            *> Scale by √dₖ = 8 AFTER full accumulation
            COMPUTE SCORE-ELEMENT(I, J) =
                SCORE-ELEMENT(I, J) / 8
        END-PERFORM
    END-PERFORM.
```

**Transpose semantics** are correct:

```
product = Q(I,K) × K(J,K)
```

because `Kᵀ(K,J) ≡ K(J,K)` under the chosen storage layout.

---

## Pipeline stages (kept separate)

```
Q × Kᵀ
   │
   ▼
scale by 1/√dₖ ← CALCULATE-ATTENTION / SCALE-ATTENTION
   │
   ▼
softmax (row-wise) ← SOFTMAX-ATTENTION
   │
   ▼
× V ← APPLY-VALUE-MATRIX
   │
   ▼
attention output
```

---

## Optional accelerator interface

```cobol
CALL "ATTN-GEMM" USING Q-VAL K-VAL SCORE
    ON EXCEPTION
        *> fall back to native COBOL reference
END-CALL
```

The external backend (HLASM, C, GPU, …) **must** implement exactly:

```
SCORE[I,J] = Σₖ Q[I,K] × K[J,K] / 8
```

It is never required for correctness.

Conceptual runtime structure:

```
COBOL PROCEDURE DIVISION
        │
        ├── native reference kernel (always present)
        │
        └── optional CALL "ATTN-GEMM"
                  │
                  ▼
          optimized native routine
                  │
          ┌───────┴────────┐
          │ │
       IBM HLASM Other accelerator
```

---

## Validation

`attention-test.cob` exercises seven deterministic cases and checks **every one of the 4 096** output elements:

1. Zero matrices  
2. Identity-compatible inputs (`Q = I`, `K = I` → `SCORE = I/8`)  
3. Constant matrices (all ones → every score = 8)  
4. Positive patterned values  
5. Negative patterned values  
6. Mixed positive/negative values  
7. Values that produce non-integer results after division by 8  

Reported for each test:

- maximum absolute error  
- minimum value  
- maximum value  
- number of mismatches (tolerance 1e-4)

---

## Hot-loop analysis & optimisation opportunities

The innermost loop

```cobol
PERFORM VARYING K FROM 1 BY 1 UNTIL K > 64
    COMPUTE SCORE-ELEMENT(I, J) =
        SCORE-ELEMENT(I, J) +
        (Q-ELEMENT(I, K) * K-ELEMENT(J, K))
END-PERFORM
```

is executed 64 × 64 × 64 = **262 144** times.

| Opportunity | Technique |
|--------------------------------|-----------|
| Contiguous memory access | Store matrices in row-major order (already true for `OCCURS`). Access `Q` row-wise and `K` row-wise for the chosen `(I,J)` pair. |
| Reduced indexing overhead | Hoist base addresses of the current Q-row and K-row outside the K loop (done automatically by good compilers; explicit in HLASM). |
| Accumulator optimi
...
