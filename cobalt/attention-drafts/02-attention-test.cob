       IDENTIFICATION DIVISION.
       PROGRAM-ID. ATTENTION-TEST.
      ******************************************************************
      * Deterministic validation driver for the COBOL attention kernel.
      * Runs four fixed test modes and reports PASS/FAIL for each.
      ******************************************************************
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. IBM-Z.
       OBJECT-COMPUTER. IBM-Z.

       DATA DIVISION.
       WORKING-STORAGE SECTION.

       01 Q-TABLE.
           05 FILLER OCCURS 64 TIMES.
               10 Q-VAL OCCURS 64 TIMES USAGE COMP-1.

       01 K-TABLE.
           05 FILLER OCCURS 64 TIMES.
               10 K-VAL OCCURS 64 TIMES USAGE COMP-1.

       01 V-TABLE.
           05 FILLER OCCURS 64 TIMES.
               10 V-VAL OCCURS 64 TIMES USAGE COMP-1.

       01 SCORE-TABLE.
           05 FILLER OCCURS 64 TIMES.
               10 SCORE OCCURS 64 TIMES USAGE COMP-1.

       01 ATTN-TABLE.
           05 FILLER OCCURS 64 TIMES.
               10 ATTENTION-OUTPUT OCCURS 64 TIMES USAGE COMP-1.

       01 I PIC 9(4) COMP.
       01 J PIC 9(4) COMP.
       01 K PIC 9(4) COMP.
       01 L PIC 9(4) COMP.

       01 C-ZERO USAGE COMP-1 VALUE 0.0.
       01 C-ONE USAGE COMP-1 VALUE 1.0.
       01 C-SQRT-DK USAGE COMP-1 VALUE 8.0.
       01 C-TOLERANCE USAGE COMP-1 VALUE 1.0E-4.
       01 C-NEG-LARGE USAGE COMP-1 VALUE -1.0E30.

       01 WS-SOFTMAX.
           05 ROW-MAX USAGE COMP-1.
           05 ROW-SUM USAGE COMP-1.
           05 EXP-VAL USAGE COMP-1.
           05 TMP-VAL USAGE COMP-1.

       01 WS-VALID.
           05 MAX-ABS-ERR USAGE COMP-1.
           05 MISMATCH-COUNT PIC 9(8) COMP.
           05 TESTS-PASSED PIC 9(4) COMP VALUE 0.
           05 TESTS-FAILED PIC 9(4) COMP VALUE 0.
           05 VALIDATION-PASSED PIC X.
               88 PASS VALUE "Y".
               88 FAIL VALUE "N".

       01 DISP-COUNT PIC Z(7)9.
       01 DISP-ERR PIC -9.9(6)E+99.

       PROCEDURE DIVISION.

       MAIN.
           DISPLAY "========================================"
           DISPLAY " COBOL ATTENTION KERNEL – TEST SUITE"
           DISPLAY "========================================"

           PERFORM TEST-ZERO
           PERFORM TEST-IDENTITY
           PERFORM TEST-CONSTANT
           PERFORM TEST-MIXED

           DISPLAY "========================================"
           MOVE TESTS-PASSED TO DISP-COUNT
           DISPLAY "PASSED: " DISP-COUNT
           MOVE TESTS-FAILED TO DISP-COUNT
           DISPLAY "FAILED: " DISP-COUNT
           DISPLAY "========================================"

           IF TESTS-FAILED > 0
               MOVE 1 TO RETURN-CODE
           ELSE
               MOVE 0 TO RETURN-CODE
           END-IF
           GOBACK.

      *-----------------------------------------------------------------
       TEST-ZERO.
           DISPLAY " "
           DISPLAY "TEST: ZERO MATRICES"
           PERFORM FILL-ZERO
           PERFORM RUN-KERNEL
           PERFORM CHECK-ALL-OUTPUT-ZERO
           PERFORM REPORT-TEST
           .

       TEST-IDENTITY.
           DISPLAY " "
           DISPLAY "TEST: IDENTITY-COMPATIBLE"
           PERFORM FILL-IDENTITY
           PERFORM RUN-KERNEL-SCORES-ONLY
           PERFORM CHECK-IDENTITY-SCORES
           PERFORM REPORT-TEST
           .

       TEST-CONSTANT.
           DISPLAY " "
           DISPLAY "TEST: CONSTANT (ALL ONES)"
           PERFORM FILL-CONSTANT
           PERFORM RUN-KERNEL-SCORES-ONLY
           PERFORM CHECK-CONSTANT-SCORES
           PERFORM REPORT-TEST
           .

       TEST-MIXED.
           DISPLAY " "
           DISPLAY "TEST: MIXED POSITIVE/NEGATIVE"

...
