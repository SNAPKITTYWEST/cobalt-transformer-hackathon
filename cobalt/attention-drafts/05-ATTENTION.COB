      ******************************************************************
      * ATTENTION.COB
      * Native COBOL Transformer Attention Kernel (reference)
      *
      * Golden reference implementation of scaled dot-product attention
      * QK^T / sqrt(d_k) for sequence length = 64, head dim = 64.
      *
      * sqrt(d_k) = 8 (exact integer scaling)
      *
      * Mathematical contract (1-based indices):
      * SCORE(I,J) = ( SUM_K Q-VAL(I,K) * K-VAL(J,K) ) / 8
      *
      * This is Q * K^T / sqrt(d_k) because K^T(K,J) == K(J,K).
      *
      * Optional optimized backend may be invoked via:
      * CALL "ATTN-GEMM" USING Q-VAL K-VAL SCORE
      * The CALL is NEVER required for correctness; the native
      * PROCEDURE DIVISION remains the authoritative implementation.
      ******************************************************************
       IDENTIFICATION DIVISION.
       PROGRAM-ID. ATTENTION.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. IBM-Z.
       OBJECT-COMPUTER. IBM-Z.

       DATA DIVISION.
       WORKING-STORAGE SECTION.

      *-----------------------------------------------------------------
      * Tensor storage (64 x 64)
      * COMP-1 = single-precision floating point (IEEE-754 on modern
      * compilers; IBM Hex FP on classic mainframe). Chosen for
      * reasonable dynamic range while remaining COBOL-standard.
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

      *-----------------------------------------------------------------
      * Softmax temporaries
      *-----------------------------------------------------------------
       01 SOFTMAX-TEMP.
           05 ROW-MAX USAGE COMP-1.
           05 ROW-SUM USAGE COMP-1.
           05 EXP-VAL USAGE COMP-1.
           05 TMP-VAL USAGE COMP-1.

      *-----------------------------------------------------------------
      * Loop indices and control
      *-----------------------------------------------------------------
       01 I PIC 9(4) COMP.
       01 J PIC 9(4) COMP.
       01 K PIC 9(4) COMP.

      *-----------------------------------------------------------------
      * Constants
      *-----------------------------------------------------------------
       01 SQRT-DK USAGE COMP-1 VALUE 8.0.
       01 ONE USAGE COMP-1 VALUE 1.0.
       01 ZERO USAGE COMP-1 VALUE 0.0.
       01 NEG-LARGE USAGE COMP-1 VALUE -1.0E30.

      *-----------------------------------------------------------------
      * Backend selection flag
      * 0 = pure COBOL reference kernel (default, always correct)
      * 1 = attempt CALL "ATTN-GEMM" (optional accelerator)
      *-----------------------------------------------------------------
       01 USE-BACKEND PIC 9 VALUE 0.

      *-----------------------------------------------------------------
      * Test / diagnostic counters (used by driver)
      *-----------------------------------------------------------------
       01 WS-DIAG.
           05 MAX-ABS-ERR USAGE COMP-1 VALUE 0.0.
           05 MIN-SCORE USAGE COMP-1 VALUE 0.0.
           05 MAX-SCORE USAGE COMP-1 VALUE 0.0.
           05 MISMATCH-COUNT PIC 9(8) COMP VALUE 0.
           05 TOTAL-ELEMENTS PIC 9(8) COMP VALUE 4096.

       PROCEDURE DIVISION.

      *
...
