      ******************************************************************
      * ATTENTION-KERNEL.COB
      * Pure native COBOL reference implementation of the scaled
      * dot-product attention score kernel.
      *
      * Callable entry points (via CALL):
      * CALCULATE-ATTENTION – QK^T / sqrt(d_k)
      * SCALE-ATTENTION – explicit scale stage
      * SOFTMAX-ATTENTION – row-wise softmax
      * APPLY-VALUE-MATRIX – softmax * V
      *
      * Mathematical contract (1-based):
      * SCORE(I,J) = ( Σ_K Q(I,K) * K(J,K) ) / 8
      *
      * Sequence length = 64, head dimension = 64, sqrt(d_k) = 8.
      ******************************************************************
       IDENTIFICATION DIVISION.
       PROGRAM-ID. ATTENTION-KERNEL.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. IBM-Z.
       OBJECT-COMPUTER. IBM-Z.

       DATA DIVISION.
       WORKING-STORAGE SECTION.

       01 I PIC 9(4) COMP.
       01 J PIC 9(4) COMP.
       01 K PIC 9(4) COMP.
       01 L PIC 9(4) COMP.

       01 SQRT-DK USAGE COMP-1 VALUE 8.0.
       01 ZERO USAGE COMP-1 VALUE 0.0.
       01 ONE USAGE COMP-1 VALUE 1.0.
       01 NEG-LARGE USAGE COMP-1 VALUE -1.0E30.

       01 SOFTMAX-TEMP.
           05 ROW-MAX USAGE COMP-1.
           05 ROW-SUM USAGE COMP-1.
           05 EXP-VAL USAGE COMP-1.
           05 TMP-VAL USAGE COMP-1.

       LINKAGE SECTION.

      *-----------------------------------------------------------------
      * All tensors are 64×64 COMP-1 tables passed by reference.
      * Layout matches the canonical declaration used by the driver.
      *-----------------------------------------------------------------
       01 Q-VAL.
           05 Q-ROW OCCURS 64 TIMES.
               10 Q-ELEMENT OCCURS 64 TIMES USAGE COMP-1.

       01 K-VAL.
           05 K-ROW OCCURS 64 TIMES.
               10 K-ELEMENT OCCURS 64 TIMES USAGE COMP-1.

       01 V-VAL.
           05 V-ROW OCCURS 64 TIMES.
               10 V-ELEMENT OCCURS 64 TIMES USAGE COMP-1.

       01 SCORE.
           05 SCORE-ROW OCCURS 64 TIMES.
               10 SCORE-ELEMENT OCCURS 64 TIMES USAGE COMP-1.

       01 ATTN-OUT.
           05 ATTN-ROW OCCURS 64 TIMES.
               10 ATTN-ELEMENT OCCURS 64 TIMES USAGE COMP-1.

       PROCEDURE DIVISION USING Q-VAL K-VAL V-VAL SCORE ATTN-OUT.

      *-----------------------------------------------------------------
      * Default entry simply returns; callers use ENTRY points below.
      *-----------------------------------------------------------------
           GOBACK.

      *=================================================================
      * CALCULATE-ATTENTION (golden reference kernel)
      *
      * Nested PERFORM VARYING is mandatory. Every product is formed
      * explicitly; the K accumulation loop is never omitted.
      *
      * Correct transpose semantics:
      * product = Q(I,K) * K(J,K)
      * because K^T(K,J) ≡ K(J,K) under the storage layout used here.
      *=================================================================
       ENTRY "CALCULATE-ATTENTION" USING Q-VAL K-VAL SCORE.

           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64

                   MOVE ZERO TO SCORE-ELEMENT(I, J)

                   PERFORM VARYING K FROM 1 BY 1 UNTIL K > 64
                       COMPUTE SCORE-ELEMENT(I, J) =
                           SCORE-ELEMENT(I, J) +
                           (Q-ELEMENT(I, K) * K-ELEMENT(J, K))
                   END-PERFORM

      * Scale AFTER the full accumulation
                   COMPUTE SCORE-ELEMENT(I, J) =
                       SCORE-ELEMENT(I, J) / SQRT-DK

               END-PERFORM
           END-PERFORM

           GOBACK.

      *====================================================
...
