      ******************************************************************
      * ATTENTION-TEST.COB
      * Deterministic validation driver for the COBOL attention kernel.
      *
      * Verifies every one of the 4 096 output elements for each case:
      * 1. zero matrices
      * 2. identity-compatible inputs
      * 3. constant matrices
      * 4. positive values
      * 5. negative values
      * 6. mixed positive/negative values
      * 7. values producing non-integer results after /8
      *
      * Reports: maximum absolute error, min/max value, mismatch count.
      ******************************************************************
       IDENTIFICATION DIVISION.
       PROGRAM-ID. ATTENTION-TEST.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. IBM-Z.
       OBJECT-COMPUTER. IBM-Z.

       DATA DIVISION.
       WORKING-STORAGE SECTION.

      *-----------------------------------------------------------------
      * Tensor storage – identical layout to the kernel
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
      * Expected reference matrix (for exact-match tests)
      *-----------------------------------------------------------------
       01 EXPECTED.
           05 EXP-ROW OCCURS 64 TIMES.
               10 EXP-ELEMENT OCCURS 64 TIMES USAGE COMP-1.

      *-----------------------------------------------------------------
      * Indices and temporaries
      *-----------------------------------------------------------------
       01 I PIC 9(4) COMP.
       01 J PIC 9(4) COMP.
       01 K PIC 9(4) COMP.

       01 ZERO USAGE COMP-1 VALUE 0.0.
       01 ONE USAGE COMP-1 VALUE 1.0.
       01 SQRT-DK USAGE COMP-1 VALUE 8.0.
       01 TOLERANCE USAGE COMP-1 VALUE 1.0E-4.

       01 WS-DIAG.
           05 MAX-ABS-ERR USAGE COMP-1 VALUE 0.0.
           05 MIN-SCORE USAGE COMP-1 VALUE 0.0.
           05 MAX-SCORE USAGE COMP-1 VALUE 0.0.
           05 MISMATCH-COUNT PIC 9(8) COMP VALUE 0.
           05 TMP-VAL USAGE COMP-1.
           05 EXPECTED-VAL USAGE COMP-1.

       01 USE-BACKEND PIC 9 VALUE 0.

       PROCEDURE DIVISION.

       MAIN.
           DISPLAY "=============================================="
           DISPLAY " COBOL Attention Kernel – Validation Suite"
           DISPLAY "=============================================="

           PERFORM TEST-ZERO
           PERFORM TEST-IDENTITY
           PERFORM TEST-CONSTANT
           PERFORM TEST-POSITIVE
           PERFORM TEST-NEGATIVE
           PERFORM TEST-MIXED
           PERFORM TEST-NONINTEGER

           DISPLAY "=============================================="
           DISPLAY " Validation complete"
           DISPLAY "=============================================="
           STOP RUN.

      *=================================================================
      * Shared helpers
      *=================================================================
       CLEAR-ALL.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   MOVE ZERO TO Q-ELEMENT(I, J)
                   MOVE ZERO TO K-ELEMENT(I, J)
                   MOVE ZERO TO V-ELEMENT(I, J)
...
