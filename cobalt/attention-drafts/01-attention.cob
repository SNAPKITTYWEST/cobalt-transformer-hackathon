       IDENTIFICATION DIVISION.
       PROGRAM-ID. ATTENTION.
      ******************************************************************
      * COBOL TRANSFORMER ATTENTION KERNEL
      * Native PROCEDURE DIVISION implementation of scaled
      * dot-product attention for sequence length 64, head dim 64.
      *
      * Kernel contract (1-based indices):
      * SCORE(I,J) = ( SUM K=1..64 Q-VAL(I,K) * K-VAL(J,K) ) / 8
      *
      * This is Q * K^T / sqrt(d_k) because K^T(K,J) = K(J,K).
      * sqrt(d_k) = 8 exactly.
      *
      * Stages:
      * CALCULATE-ATTENTION
      * SOFTMAX-ATTENTION
      * APPLY-VALUE-MATRIX
      ******************************************************************
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. IBM-Z WITH DEBUGGING MODE.
       OBJECT-COMPUTER. IBM-Z.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT OPTIONAL TEST-VECTOR-FILE
               ASSIGN TO "data/test-vectors.dat"
               ORGANIZATION IS LINE SEQUENTIAL
               FILE STATUS IS WS-FILE-STATUS.

       DATA DIVISION.
       FILE SECTION.
       FD TEST-VECTOR-FILE.
       01 TEST-VECTOR-RECORD PIC X(256).

       WORKING-STORAGE SECTION.

      *-----------------------------------------------------------------
      * 64 x 64 tensors (USAGE COMP-1 = single-precision floating
      * point; supported by GnuCOBOL, IBM Enterprise COBOL, Micro Focus)
      *-----------------------------------------------------------------
       01 Q-VAL.
           05 Q-ROW OCCURS 64 TIMES.
               10 Q-VAL OCCURS 64 TIMES USAGE COMP-1.

       01 K-VAL.
           05 K-ROW OCCURS 64 TIMES.
               10 K-VAL OCCURS 64 TIMES USAGE COMP-1.

       01 V-VAL.
           05 V-ROW OCCURS 64 TIMES.
               10 V-VAL OCCURS 64 TIMES USAGE COMP-1.

       01 SCORE.
           05 SCORE-ROW OCCURS 64 TIMES.
               10 SCORE OCCURS 64 TIMES USAGE COMP-1.

       01 ATTENTION-OUTPUT.
           05 ATTN-ROW OCCURS 64 TIMES.
               10 ATTENTION-OUTPUT OCCURS 64 TIMES USAGE COMP-1.

      *-----------------------------------------------------------------
      * Softmax working storage
      *-----------------------------------------------------------------
       01 WS-SOFTMAX.
           05 ROW-MAX USAGE COMP-1.
           05 ROW-SUM USAGE COMP-1.
           05 EXP-VAL USAGE COMP-1.
           05 TMP-VAL USAGE COMP-1.

      *-----------------------------------------------------------------
      * Indexes
      *-----------------------------------------------------------------
       01 I PIC 9(4) COMP.
       01 J PIC 9(4) COMP.
       01 K PIC 9(4) COMP.
       01 L PIC 9(4) COMP.

      *-----------------------------------------------------------------
      * Constants (explicit, not magic)
      *-----------------------------------------------------------------
       01 C-SEQ-LEN PIC 9(4) COMP VALUE 64.
       01 C-HEAD-DIM PIC 9(4) COMP VALUE 64.
       01 C-SQRT-DK USAGE COMP-1 VALUE 8.0.
       01 C-ZERO USAGE COMP-1 VALUE 0.0.
       01 C-ONE USAGE COMP-1 VALUE 1.0.
       01 C-NEG-LARGE USAGE COMP-1 VALUE -1.0E30.
       01 C-TOLERANCE USAGE COMP-1 VALUE 1.0E-4.

      *-----------------------------------------------------------------
      * Validation / diagnostics
      *-----------------------------------------------------------------
       01 WS-VALID.
           05 MAX-ABS-ERR USAGE COMP-1 VALUE 0.0.
           05 MIN-SCORE USAGE COMP-1 VALUE 0.0.
           05 MAX-SCORE USAGE COMP-1 VALUE 0.0.
           05 MISMATCH-COUNT PIC 9(8) COMP VALUE 0.
           05 TOTAL-ELEMENTS PIC 9(8) COMP VALUE 4096.
           05 SOFTMAX-ROW-ERR
...
