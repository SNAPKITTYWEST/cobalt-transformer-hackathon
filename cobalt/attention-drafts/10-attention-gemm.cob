       IDENTIFICATION DIVISION.
       PROGRAM-ID. ATTENTION.
      ******************************************************************
      * COBOL 2023 / 2024 TRANSFORMER ATTENTION KERNEL
      * ----------------------------------------------------------
      * Native PROCEDURE DIVISION reference implementation of
      * scaled dot-product attention expressed as an explicit GEMM:
      *
      * SCORE = GEMM(Q, K^T) / sqrt(d_k)
      *
      * where
      * GEMM(A,B)[I,J] = SUM_K A[I,K] * B[K,J]
      * and because we store K row-major,
      * K^T[K,J] = K[J,K]
      * therefore the inner product is
      * Q-VAL(I,K) * K-VAL(J,K)
      *
      * Dimensions (explicit, not hidden):
      * sequence length = 64
      * head dimension = 64
      * sqrt(d_k) = 8
      *
      * Pipeline (each stage is a real COBOL paragraph):
      * CALCULATE-ATTENTION (= GEMM + scale)
      * SOFTMAX-ATTENTION
      * APPLY-VALUE-MATRIX (= another GEMM)
      *
      * Optional accelerator interface (never required):
      * CALL "ATTN-GEMM" USING Q-TABLE K-TABLE SCORE-TABLE
      *
      * A Triton / CUDA / HLASM backend may implement the identical
      * mathematical contract; the native COBOL loops remain the
      * golden reference. True Triton kernels are written in Python
      * and cannot replace this PROCEDURE DIVISION.
      ******************************************************************
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. IBM-Z.
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
      * Tensor storage – 64 x 64 COMP-1
      * Elementary names chosen so the required subscript form is
      * legal and unambiguous:
      * Q-VAL(I, K) K-VAL(J, K) SCORE(I, J)
      * V-VAL(J, L) ATTENTION-OUTPUT(I, L)
      *-----------------------------------------------------------------
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

      *-----------------------------------------------------------------
      * Softmax temporaries
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
      * Explicit constants
      *-----------------------------------------------------------------
       01 C-SEQ-LEN PIC 9(4) COMP VALUE 64.
       01 C-HEAD-DIM PIC 9(4) COMP VALUE 64.
       01 C-SQRT-DK USAGE COMP-1 VALUE 8.0.
       01 C-ZERO USAGE COMP-1 VALUE 0.0.
       01 C-ONE USAGE COMP-1 VALUE 1.0.
       01 C-NEG-LARGE USAGE COMP-1 VALUE -1.0E30.
       01 C-TOLERANCE USAGE COMP-1 VALUE 1.0E-4.

      *-----------------------------------------------------------------
      * Control & diagnostics
      *-----------------------------------------------------------------
       01 WS-VALID.
           05 MAX-ABS-ERR USAGE COMP-1 VALUE 0.0.
           05 MIN-SCORE USAGE COMP-1 VALUE 0.0.
           05 MAX-SCORE USAGE COMP-1 VALUE 0.0.
           05 MISMATCH-COUNT PIC 9(8) COMP VALUE 0.
           05 TOTAL-ELEMENTS PIC 9(8) COMP VALUE 4096.
           05 SOFTMAX-ROW-ERR USAGE COMP-1 VALUE 0.0.
           05 VALIDATION-PASSED PIC X VALUE "Y".
               88 PASS VALUE "Y".
               88 FAIL VALUE "N".

       01 WS-FILE-STATUS PIC XX VALUE "00".
       01 WS-TEST-MODE PIC X VALUE "I".
               88 MODE-IDENTITY VALUE "I".
               88 MODE-ZERO VALUE "Z".
               88 MODE-CONSTANT VALUE "C".
               88 MODE-MIXED VALUE "M".
               88 MODE-EXTERNAL VALUE "E".

       01 USE-BACKEND PIC 9 VALUE 0.
               88 BACKEND-ON VALUE 1.
               88 BACKEND-OFF VALUE 0.

       01 WS-DISPLAY.
           05 DISP-I PIC ZZZ9.
           05 DISP-J PIC ZZZ9.
           05 DISP-VAL PIC -9.9(6)E+99.
           05 DISP-COUNT PIC Z(7)9.
           05 DISP-ERR PIC -9.9(6)E+99.

       PROCEDURE DIVISION.

      *=================================================================
      * MAIN
      *=================================================================
       MAIN-PROGRAM.
           DISPLAY "KERNEL: COBOL ATTENTION (GEMM form)"
           DISPLAY "DIMENSION: 64x64"
           DISPLAY "HEAD-DIMENSION: 64"
           DISPLAY "SCALE: 8"
           DISPLAY "BACKEND: NATIVE COBOL REFERENCE"

           PERFORM INITIALIZE-Q
           PERFORM INITIALIZE-K
           PERFORM INITIALIZE-V
           PERFORM CLEAR-SCORE
           PERFORM CLEAR-ATTENTION-OUTPUT

           PERFORM CALCULATE-ATTENTION
           PERFORM SOFTMAX-ATTENTION
           PERFORM APPLY-VALUE-MATRIX

           PERFORM VALIDATE-KERNEL
           PERFORM DISPLAY-RESULTS

           IF FAIL
               MOVE 1 TO RETURN-CODE
           ELSE
               MOVE 0 TO RETURN-CODE
           END-IF
           GOBACK.

      *=================================================================
      * INITIALIZATION
      *=================================================================
       INITIALIZE-Q.
           EVALUATE TRUE
               WHEN MODE-ZERO
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           MOVE C-ZERO TO Q-VAL(I, J)
                       END-PERFORM
                   END-PERFORM
               WHEN MODE-IDENTITY
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           IF I = J
                               MOVE C-ONE TO Q-VAL(I, J)
                           ELSE
                               MOVE C-ZERO TO Q-VAL(I, J)
                           END-IF
                       END-PERFORM
                   END-PERFORM
               WHEN MODE-CONSTANT
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           MOVE C-ONE TO Q-VAL(I, J)
                       END-PERFORM
                   END-PERFORM
               WHEN MODE-MIXED
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           COMPUTE Q-VAL(I, J) =
                               FUNCTION MOD(I + J, 11) - 5
                       END-PERFORM
                   END-PERFORM
               WHEN OTHER
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           IF I = J
                               MOVE C-ONE TO Q-VAL(I, J)
                           ELSE
                               MOVE C-ZERO TO Q-VAL(I, J)
                           END-IF
                       END-PERFORM
                   END-PERFORM
           END-EVALUATE
           .

       INITIALIZE-K.
           EVALUATE TRUE
               WHEN MODE-ZERO
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           MOVE C-ZERO TO K-VAL(I, J)
                       END-PERFORM
                   END-PERFORM
               WHEN MODE-IDENTITY
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           IF I = J
                               MOVE C-ONE TO K-VAL(I, J)
                           ELSE
                               MOVE C-ZERO TO K-VAL(I, J)
                           END-IF
                       END-PERFORM
                   END-PERFORM
               WHEN MODE-CONSTANT
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           MOVE C-ONE TO K-VAL(I, J)
                       END-PERFORM
                   END-PERFORM
               WHEN MODE-MIXED
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           COMPUTE K-VAL(I, J) =
                               FUNCTION MOD(I * 2 + J, 9) - 4
                       END-PERFORM
                   END-PERFORM
               WHEN OTHER
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           IF I = J
                               MOVE C-ONE TO K-VAL(I, J)
                           ELSE
                               MOVE C-ZERO TO K-VAL(I, J)
                           END-IF
                       END-PERFORM
                   END-PERFORM
           END-EVALUATE
           .

       INITIALIZE-V.
           EVALUATE TRUE
               WHEN MODE-ZERO
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           MOVE C-ZERO TO V-VAL(I, J)
                       END-PERFORM
                   END-PERFORM
               WHEN MODE-IDENTITY
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           IF I = J
                               MOVE C-ONE TO V-VAL(I, J)
                           ELSE
                               MOVE C-ZERO TO V-VAL(I, J)
                           END-IF
                       END-PERFORM
                   END-PERFORM
               WHEN MODE-CONSTANT
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           MOVE C-ONE TO V-VAL(I, J)
                       END-PERFORM
                   END-PERFORM
               WHEN MODE-MIXED
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           COMPUTE V-VAL(I, J) =
                               FUNCTION MOD(I + J * 3, 7) - 3
                       END-PERFORM
                   END-PERFORM
               WHEN OTHER
                   PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
                       PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                           IF I = J
                               MOVE C-ONE TO V-VAL(I, J)
                           ELSE
                               MOVE C-ZERO TO V-VAL(I, J)
                           END-IF
                       END-PERFORM
                   END-PERFORM
           END-EVALUATE
           .

       CLEAR-SCORE.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   MOVE C-ZERO TO SCORE(I, J)
               END-PERFORM
           END-PERFORM
           .

       CLEAR-ATTENTION-OUTPUT.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   MOVE C-ZERO TO ATTENTION-OUTPUT(I, J)
               END-PERFORM
           END-PERFORM
           .

      *=================================================================
      * GEMM-FORM ATTENTION KERNEL (golden reference)
      *
      * This is the required computation. It is a GEMM:
      * C = alpha * A * B^T with alpha = 1/sqrt(d_k) = 1/8
      *
      * The triple nested PERFORM is intentional and mandatory.
      * No BLAS, no Triton, no Python, no external library replaces it.
      *=================================================================
       CALCULATE-ATTENTION.
           IF BACKEND-ON
               PERFORM TRY-BACKEND-GEMM
           ELSE
               PERFORM NATIVE-GEMM-ATTENTION
           END-IF
           .

       NATIVE-GEMM-ATTENTION.
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
           END-PERFORM
           .

      *-----------------------------------------------------------------
      * Optional CALL interface for a pre-built GEMM / Triton /
      * HLASM / other accelerator. The backend MUST implement
      * exactly the same mathematics. On any failure control
      * returns to the native COBOL reference.
      *-----------------------------------------------------------------
       TRY-BACKEND-GEMM.
           CALL "ATTN-GEMM" USING
               Q-TABLE
               K-TABLE
               SCORE-TABLE
               ON EXCEPTION
                   DISPLAY "ATTN-GEMM absent – native COBOL GEMM used"
                   SET BACKEND-OFF TO TRUE
                   PERFORM NATIVE-GEMM-ATTENTION
           END-CALL
           .

      *=================================================================
      * SOFTMAX (stable, row-wise)
      *=================================================================
       SOFTMAX-ATTENTION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               MOVE C-NEG-LARGE TO ROW-MAX
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   IF SCORE(I, J) > ROW-MAX
                       MOVE SCORE(I, J) TO ROW-MAX
                   END-IF
               END-PERFORM

               MOVE C-ZERO TO ROW-SUM
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   COMPUTE TMP-VAL = SCORE(I, J) - ROW-MAX
                   COMPUTE EXP-VAL = FUNCTION EXP(TMP-VAL)
                   MOVE EXP-VAL TO SCORE(I, J)
                   ADD EXP-VAL TO ROW-SUM
               END-PERFORM

               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   IF ROW-SUM NOT = C-ZERO
                       COMPUTE SCORE(I, J) =
                           SCORE(I, J) / ROW-SUM
                   ELSE
                       MOVE C-ZERO TO SCORE(I, J)
                   END-IF
               END-PERFORM
           END-PERFORM
           .

      *=================================================================
      * APPLY-VALUE-MATRIX (second GEMM: softmax(QK^T) * V)
      *=================================================================
       APPLY-VALUE-MATRIX.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING L FROM 1 BY 1 UNTIL L > 64
                   MOVE C-ZERO TO ATTENTION-OUTPUT(I, L)
                   PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                       COMPUTE ATTENTION-OUTPUT(I, L) =
                           ATTENTION-OUTPUT(I, L) +
                           (SCORE(I, J) * V-VAL(J, L))
                   END-PERFORM
               END-PERFORM
           END-PERFORM
           .

      *=================================================================
      * VALIDATION
      *=================================================================
       VALIDATE-KERNEL.
           SET PASS TO TRUE
           MOVE C-ZERO TO MAX-ABS-ERR
           MOVE 0 TO MISMATCH-COUNT
           MOVE C-ZERO TO SOFTMAX-ROW-ERR

           EVALUATE TRUE
               WHEN MODE-ZERO
                   PERFORM VALIDATE-ZERO
               WHEN MODE-IDENTITY
                   PERFORM VALIDATE-IDENTITY
               WHEN MODE-CONSTANT
                   PERFORM VALIDATE-CONSTANT
               WHEN MODE-MIXED
                   PERFORM VALIDATE-MIXED
               WHEN OTHER
                   PERFORM VALIDATE-IDENTITY
           END-EVALUATE

           PERFORM VALIDATE-SOFTMAX-ROWS
           PERFORM VALIDATE-FINITE
           .

       VALIDATE-ZERO.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   COMPUTE TMP-VAL =
                       FUNCTION ABS(ATTENTION-OUTPUT(I, J))
                   IF TMP-VAL > C-TOLERANCE
                       ADD 1 TO MISMATCH-COUNT
                       SET FAIL TO TRUE
                   END-IF
                   IF TMP-VAL > MAX-ABS-ERR
                       MOVE TMP-VAL TO MAX-ABS-ERR
                   END-IF
               END-PERFORM
           END-PERFORM
           .

       VALIDATE-IDENTITY.
           PERFORM CLEAR-SCORE
           PERFORM NATIVE-GEMM-ATTENTION
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   IF I = J
                       COMPUTE TMP-VAL =
                           SCORE(I, J) - (C-ONE / C-SQRT-DK)
                   ELSE
                       MOVE SCORE(I, J) TO TMP-VAL
                   END-IF
                   COMPUTE TMP-VAL = FUNCTION ABS(TMP-VAL)
                   IF TMP-VAL > C-TOLERANCE
                       ADD 1 TO MISMATCH-COUNT
                       SET FAIL TO TRUE
                   END-IF
                   IF TMP-VAL > MAX-ABS-ERR
                       MOVE TMP-VAL TO MAX-ABS-ERR
                   END-IF
               END-PERFORM
           END-PERFORM
           .

       VALIDATE-CONSTANT.
           PERFORM CLEAR-SCORE
           PERFORM NATIVE-GEMM-ATTENTION
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   COMPUTE TMP-VAL =
                       FUNCTION ABS(SCORE(I, J) - 8.0)
                   IF TMP-VAL > C-TOLERANCE
                       ADD 1 TO MISMATCH-COUNT
                       SET FAIL TO TRUE
                   END-IF
                   IF TMP-VAL > MAX-ABS-ERR
                       MOVE TMP-VAL TO MAX-ABS-ERR
                   END-IF
               END-PERFORM
           END-PERFORM
           .

       VALIDATE-MIXED.
           PERFORM CLEAR-SCORE
           PERFORM NATIVE-GEMM-ATTENTION
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   MOVE C-ZERO TO TMP-VAL
                   PERFORM VARYING K FROM 1 BY 1 UNTIL K > 64
                       COMPUTE TMP-VAL =
                           TMP-VAL +
                           (Q-VAL(I, K) * K-VAL(J, K))
                   END-PERFORM
                   COMPUTE TMP-VAL = TMP-VAL / C-SQRT-DK
                   COMPUTE TMP-VAL =
                       FUNCTION ABS(TMP-VAL - SCORE(I, J))
                   IF TMP-VAL > C-TOLERANCE
                       ADD 1 TO MISMATCH-COUNT
                       SET FAIL TO TRUE
                   END-IF
                   IF TMP-VAL > MAX-ABS-ERR
                       MOVE TMP-VAL TO MAX-ABS-ERR
                   END-IF
               END-PERFORM
           END-PERFORM
           .

       VAL
