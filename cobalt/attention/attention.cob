      *> =====================================================================
      *> attention.cob — Transformer attention kernel, native COBOL reference
      *> implementation (the golden reference kernel).
      *>
      *> Computes, for 64x64 matrices (seq len = 64, head dim = 64):
      *>
      *> SCORE[I,J] = ( SUM(K=1..64) Q[I,K] * K[J,K] ) / 8
      *>
      *> i.e. SCORE = (Q x K^T) / sqrt(d_k), with d_k = 64 and sqrt(64) = 8.
      *> Note the transpose semantics: the accumulation multiplies Q(I,K) by
      *> K(J,K) — K is addressed by (row=J, col=K), NOT by (K,J).
      *>
      *> Numeric representation: COMP-2 (double-precision float) is used for
      *> Q, K, SCORE and all accumulators. Rationale: the inner product of
      *> 64 products can reach magnitudes where COMP-1 (single precision)
      *> would lose several digits; COMP-2 keeps the reference exact for the
      *> test corpus and near-exact in general. On z/OS COMP-2 is hexadecimal
      *> floating point; on GnuCOBOL it is IEEE binary64 — either way the
      *> semantics below are identical.
      *>
      *> Compiler compatibility: written for GnuCOBOL (cobc) and IBM Enterprise
      *> COBOL. Uses only standard COBOL 85/2002 features plus the standard
      *> intrinsic functions EXP / ABS / MOD / TRIM.
      *>
      *> Structure of this file (two separately callable subprograms):
      *> ATTN-CALC — the reference kernel: SCORE = (Q x K^T)/8
      *> (paragraphs CALCULATE-ATTENTION, SCALE-ATTENTION)
      *> ATTN-PIPE — the full pipeline:
      *> SCORE = (Q x K^T)/8 -> softmax per row -> x V -> OUT
      *> (paragraphs CALCULATE-ATTENTION, SCALE-ATTENTION,
      *> SOFTMAX-ATTENTION, APPLY-VALUE-MATRIX)
      *>
      *> Optional accelerator: ATTN-CALC consults ATTN-USE-BACKEND. When the
      *> flag is 'Y' it CALLs "ATTN-GEMM"; if the entry point does not exist
      *> (ON EXCEPTION) it silently falls back to the native kernel, so the
      *> external backend is NEVER mandatory for correctness.
      *> =====================================================================

       IDENTIFICATION DIVISION.
       PROGRAM-ID. ATTN-CALC.
      *> Reference kernel: SCORE = (Q x K^T) / 8, 64x64.

       DATA DIVISION.

       WORKING-STORAGE SECTION.
      *> Loop indices: binary (COMP-5) so subscripting uses hardware
      *> registers — the native loop is exactly the specified triple loop.
       01 WS-IND.
           05 I PIC S9(4) COMP-5 VALUE 0.
           05 J PIC S9(4) COMP-5 VALUE 0.
           05 K PIC S9(4) COMP-5 VALUE 0.

      *> Backend switch. 'N' (default) = native COBOL only.
      *> The driver may set it to 'Y'; a missing backend degrades gracefully.
       01 ATTN-USE-BACKEND PIC X VALUE 'N'.

       LINKAGE SECTION.
      *> Logical 64x64 tensors. Elementary items carry the OCCURS so the
      *> storage is 64*64*8 bytes, row-major, contiguous per row.
       01 Q-VAL.
           05 Q-ROW OCCURS 64 TIMES.
               10 Q-ELEMENT USAGE COMP-2 OCCURS 64 TIMES.
       01 K-VAL.
           05 K-ROW OCCURS 64 TIMES.
               10 K-ELEMENT USAGE COMP-2 OCCURS 64 TIMES.
       01 SCORE.
           05 SCORE-ROW OCCURS 64 TIMES.
               10 SCORE-ELEMENT USAGE COMP-2 OCCURS 64 TIMES.

       PROCEDURE DIVISION USING Q-VAL K-VAL SCORE.

      *> ------------------------------------------------------------------
      *> CALCULATE-ATTENTION — golden reference kernel.
      *> Loop order I (rows of Q) / J (rows of K) / K (contraction).
      *> Accumulation happens BEFORE scaling: SCORE is zeroed, all 64
      *> products are accumulated, and only then is the element divided
      *> by 8 (= sqrt(d_k)). Scaling is never inside the K loop.
      *> ------------------------------------------------------------------
       CALCULATE-ATTENTION.
           IF ATTN-USE-BACKEND = 'Y'
               CALL "ATTN-GEMM" USING Q-VAL K-VAL SCORE
                   ON EXCEPTION
                       MOVE 'N' TO ATTN-USE-BACKEND
               NOT ON EXCEPTION
                   GOBACK
           END-IF

           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   MOVE 0 TO SCORE-ELEMENT (I J)
                   PERFORM VARYING K FROM 1 BY 1 UNTIL K > 64
                       COMPUTE SCORE-ELEMENT (I J) =
                           SCORE-ELEMENT (I J) +
                           (Q-ELEMENT (I K) * K-ELEMENT (J K))
                   END-PERFORM
      *> Scale by sqrt(d_k): d_k = 64, therefore sqrt(d_k) = 8.
                   COMPUTE SCORE-ELEMENT (I J) =
                       SCORE-ELEMENT (I J) / 8
               END-PERFORM
           END-PERFORM
           .

      *> ------------------------------------------------------------------
      *> SCALE-ATTENTION — separate, explicit 1/sqrt(d_k) scaling stage.
      *> Kept as its own paragraph so an unnormalized product can be
      *> produced and scaled in a second pass if ever needed; the kernel
      *> above calls it inline-equivalently via the trailing COMPUTE.
      *> ------------------------------------------------------------------
       SCALE-ATTENTION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   COMPUTE SCORE-ELEMENT (I J) =
                       SCORE-ELEMENT (I J) / 8
               END-PERFORM
           END-PERFORM
           .

       END PROGRAM ATTN-CALC.
      *> =====================================================================


      *> =====================================================================
      *> ATTN-PIPE — complete attention pipeline as four distinct stages:
      *> 1. CALCULATE-ATTENTION : S = Q x K^T (raw product, unscaled)
      *> 2. SCALE-ATTENTION : SCORE = S / 8 — the one explicit scale
      *> 3. SOFTMAX-ATTENTION : PROB(I,J) = exp(S(I,J)-m_I)/Z_I,
      *> max-shifted per row for stability
      *> 4. APPLY-VALUE-MATRIX : OUT(I,D) = SUM(J) PROB(I,J)*V(J,D)
      *> The stages are NOT collapsed into one opaque procedure.
      *> =====================================================================
       IDENTIFICATION DIVISION.
       PROGRAM-ID. ATTN-PIPE.

       DATA DIVISION.

       WORKING-STORAGE SECTION.
       01 WS-IND.
           05 I PIC S9(4) COMP-5 VALUE 0.
           05 J PIC S9(4) COMP-5 VALUE 0.
           05 K PIC S9(4) COMP-5 VALUE 0.
           05 D PIC S9(4) COMP-5 VALUE 0.
       01 WS-SCALARS.
           05 WS-MAX USAGE COMP-2 VALUE 0.
           05 WS-SUM USAGE COMP-2 VALUE 0.
           05 WS-E USAGE COMP-2 VALUE 0.

       LINKAGE SECTION.
       01 Q-VAL.
           05 Q-ROW OCCURS 64 TIMES.
               10 Q-ELEMENT USAGE COMP-2 OCCURS 64 TIMES.
       01 K-VAL.
           05 K-ROW OCCURS 64 TIMES.
               10 K-ELEMENT USAGE COMP-2 OCCURS 64 TIMES.
       01 V-VAL.
           05 V-ROW OCCURS 64 TIMES.
               10 V-ELEMENT USAGE COMP-2 OCCURS 64 TIMES.
       01 SCORE.
           05 SCORE-ROW OCCURS 64 TIMES.
               10 SCORE-ELEMENT USAGE COMP-2 OCCURS 64 TIMES.
       01 PROB.
           05 PROB-ROW OCCURS 64 TIMES.
               10 PROB-ELEMENT USAGE COMP-2 OCCURS 64 TIMES.
       01 OUT-MAT.
           05 OUT-ROW OCCURS 64 TIMES.
               10 OUT-ELEMENT USAGE COMP-2 OCCURS 64 TIMES.

       PROCEDURE DIVISION
               USING Q-VAL K-VAL V-VAL SCORE PROB OUT-MAT.

      *> Stage 1: raw product S = Q x K^T (NO scaling here — stage 2 is the
      *> separate scale stage, so the pipeline applies exactly one 1/8).
      *> The standalone kernel ATTN-CALC fuses this scale into its trailing
      *> COMPUTE exactly as the required kernel specifies; the pipeline
      *> keeps it as an explicit, visible stage.
       CALCULATE-ATTENTION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   MOVE 0 TO SCORE-ELEMENT (I J)
                   PERFORM VARYING K FROM 1 BY 1 UNTIL K > 64
                       COMPUTE SCORE-ELEMENT (I J) =
                           SCORE-ELEMENT (I J) +
                           (Q-ELEMENT (I K) * K-ELEMENT (J K))
                   END-PERFORM
               END-PERFORM
           END-PERFORM
           .

      *> Stage 2: scaling by 1/sqrt(d_k) = 1/8. Applied exactly once.
       SCALE-ATTENTION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   COMPUTE SCORE-ELEMENT (I J) =
                       SCORE-ELEMENT (I J) / 8
               END-PERFORM
           END-PERFORM
           .

      *> Stage 3: numerically stable per-row softmax.
       SOFTMAX-ATTENTION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
      *> row max for shift
               MOVE SCORE-ELEMENT (I 1) TO WS-MAX
               PERFORM VARYING J FROM 2 BY 1 UNTIL J > 64
                   IF SCORE-ELEMENT (I J) > WS-MAX
                       MOVE SCORE-ELEMENT (I J) TO WS-MAX
                   END-IF
               END-PERFORM
      *> exponentials against the max
               MOVE 0 TO WS-SUM
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   COMPUTE WS-E =
                       FUNCTION EXP (SCORE-ELEMENT (I J) - WS-MAX)
                   MOVE WS-E TO PROB-ELEMENT (I J)
                   COMPUTE WS-SUM = WS-SUM + WS-E
               END-PERFORM
      *> normalize
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   COMPUTE PROB-ELEMENT (I J) =
                       PROB-ELEMENT (I J) / WS-SUM
               END-PERFORM
           END-PERFORM
           .

      *> Stage 4: apply the value matrix — OUT(I,D) = SUM J PROB(I,J)*V(J,D).
       APPLY-VALUE-MATRIX.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING D FROM 1 BY 1 UNTIL D > 64
                   MOVE 0 TO OUT-ELEMENT (I D)
                   PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                       COMPUTE OUT-ELEMENT (I D) =
                           OUT-ELEMENT (I D) +
                           (PROB-ELEMENT (I J) * V-ELEMENT (J D))
                   END-PERFORM
               END-PERFORM
           END-PERFORM
           .

      *> Pipeline driver: stages in order, never fused.
       RUN-PIPE.
           PERFORM CALCULATE-ATTENTION
           PERFORM SCALE-ATTENTION
           PERFORM SOFTMAX-ATTENTION
           PERFORM APPLY-VALUE-MATRIX
           .

       END PROGRAM ATTN-PIPE.
