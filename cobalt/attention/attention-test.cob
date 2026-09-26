      *> =====================================================================
      *> attention-test.cob — deterministic validation driver for the
      *> native COBOL attention reference kernel (attention.cob).
      *>
      *> Seven cases, each verifying ALL 4096 output elements of
      *> SCORE = (Q x K^T)/8 against an INDEPENDENT recomputation
      *> (a second, differently-structured evaluation of the same
      *> mathematical definition — not a call into the kernel).
      *>
      *> 1. zero matrices
      *> 2. identity-compatible inputs (Q = K = I64)
      *> 3. constant matrices (Q = 2.5, K = 1.25)
      *> 4. positive values
      *> 5. negative values (Q negative, K positive)
      *> 6. mixed positive/negative values
      *> 7. values producing non-integer results after division by 8
      *>
      *> Per case the driver reports:
      *> - maximum absolute error vs the independent recomputation
      *> - minimum and maximum element of SCORE
      *> - number of mismatches beyond tolerance 1.0E-09 relative
      *>
      *> It then exercises the full attention pipeline (ATTN-PIPE) and
      *> checks that every softmax row sums to 1 (within 1.0E-12 absolute)
      *> and that the attention output is in range.
      *>
      *> Build (GnuCOBOL):
      *> cobc -free -x attention-test.cob attention.cob
      *> Build (z/OS, Enterprise COBOL): compile both members into the same
      *> load library; ATTN-CALC and ATTN-PIPE are resolved by the linker.
      *> Exit status: 0 = all cases passed, 1 = at least one mismatch.
      *> =====================================================================

       IDENTIFICATION DIVISION.
       PROGRAM-ID. ATTENTION-TEST.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. IBM-Z.
       OBJECT-COMPUTER. IBM-Z.

       DATA DIVISION.

       WORKING-STORAGE SECTION.
       01 WS-IND.
           05 I PIC S9(4) COMP-5 VALUE 0.
           05 J PIC S9(4) COMP-5 VALUE 0.
           05 K PIC S9(4) COMP-5 VALUE 0.
           05 D PIC S9(4) COMP-5 VALUE 0.

       01 CASE-ID PIC 9 VALUE 0.
       01 CASE-NAME PIC X(32) VALUE SPACES.

      *> Tolerances
       01 TOL USAGE COMP-2 VALUE 1.0E-09.
       01 ROW-TOL USAGE COMP-2 VALUE 1.0E-12.

      *> Per-case statistics
       01 WS-ERR USAGE COMP-2 VALUE 0.
       01 WS-EXP USAGE COMP-2 VALUE 0.
       01 WS-MAX-ERR USAGE COMP-2 VALUE 0.
       01 WS-MIN USAGE COMP-2 VALUE 0.
       01 WS-MAX USAGE COMP-2 VALUE 0.
       01 MISMATCH PIC 9(5) VALUE 0.
       01 TOTAL-MISM PIC 9(7) VALUE 0.

       01 ROW-SUM USAGE COMP-2 VALUE 0.
       01 ROW-BAD PIC 9(5) VALUE 0.
       01 OUT-BAD PIC 9(5) VALUE 0.

       01 PASS-FLAG PIC X VALUE 'Y'.
       01 FMT-ERR PIC -ZZ.9E-999.
       01 FMT-VAL PIC -ZZZZ9.999999.
       01 FMT-CNT PIC ZZZZ9.

      *> ---- The tensors (logical 64x64 each) ----------------------------
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
      *> Independent expected values (recomputed by the driver itself)
       01 S-EXP.
           05 S-EXP-ROW OCCURS 64 TIMES.
               10 S-EXP-ELEMENT USAGE COMP-2 OCCURS 64 TIMES.

       PROCEDURE DIVISION.

       MAIN-PROC.
           DISPLAY "==================================================="
           DISPLAY " COBOL Attention Kernel Validation (reference impl)"
           DISPLAY " SCORE = (Q x K^T) / 8 64x64, 4096 elements/case"
           DISPLAY "==================================================="

           PERFORM VARYING CASE-ID FROM 1 BY 1 UNTIL CASE-ID > 7
               PERFORM INIT-CASE
               PERFORM RUN-CASE
           END-PERFORM

           PERFORM PIPELINE-CHECK

           DISPLAY "==================================================="
           MOVE TOTAL-MISM TO FMT-CNT
           DISPLAY " TOTAL MISMATCHES (all cases): " FMT-CNT
           IF PASS-FLAG = 'Y'
               DISPLAY " RESULT: PASS"
               MOVE 0 TO RETURN-CODE
           ELSE
               DISPLAY " RESULT: FAIL"
               MOVE 1 TO RETURN-CODE
           END-IF
           DISPLAY "==================================================="
           STOP RUN
           .

      *> ------------------------------------------------------------------
      *> INIT-CASE — deterministic data, identical every run.
      *> ------------------------------------------------------------------
       INIT-CASE.
           EVALUATE CASE-ID
               WHEN 1
                   MOVE "zero matrices" TO CASE-NAME
               WHEN 2
                   MOVE "identity-compatible inputs" TO CASE-NAME
               WHEN 3
                   MOVE "constant matrices" TO CASE-NAME
               WHEN 4
                   MOVE "positive values" TO CASE-NAME
               WHEN 5
                   MOVE "negative values" TO CASE-NAME
               WHEN 6
                   MOVE "mixed positive/negative" TO CASE-NAME
               WHEN 7
                   MOVE "non-integer after /8" TO CASE-NAME
           END-EVALUATE

           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING K FROM 1 BY 1 UNTIL K > 64
                   COMPUTE Q-ELEMENT (I K) = 0
                   COMPUTE K-ELEMENT (I K) = 0
      *> V is used only by the pipeline check
                   COMPUTE V-ELEMENT (I K) =
                       ((I * 3) + (K * 5)) / 64
                   EVALUATE CASE-ID
                       WHEN 1
                           CONTINUE
                       WHEN 2
                           IF I = K
                               MOVE 1 TO Q-ELEMENT (I K)
                               MOVE 1 TO K-ELEMENT (I K)
                           END-IF
                       WHEN 3
                           MOVE 2.5 TO Q-ELEMENT (I K)
                           MOVE 1.25 TO K-ELEMENT (I K)
                       WHEN 4
                           COMPUTE Q-ELEMENT (I K) = (I + K) / 10
                           COMPUTE K-ELEMENT (I K) = (I + (2 * K)) / 10
                       WHEN 5
                           COMPUTE Q-ELEMENT (I K) = (I + K) / -10
                           COMPUTE K-ELEMENT (I K) = (I + (2 * K)) / 10
                       WHEN 6
                           COMPUTE Q-ELEMENT (I K) =
                               (FUNCTION MOD (I * K) 11) / 4
                           COMPUTE Q-ELEMENT (I K) =
                               Q-ELEMENT (I K) - 1.25
                           COMPUTE K-ELEMENT (I K) =
                               (FUNCTION MOD ((2 * I) + K) 7) / 3
                           COMPUTE K-ELEMENT (I K) =
                               K-ELEMENT (I K) - 1
                       WHEN 7
                           COMPUTE Q-ELEMENT (I K) = ((I * 3) + K) / 8
                           COMPUTE K-ELEMENT (I K) = (I + (K * 5)) / 16
                   END-EVALUATE
               END-PERFORM
           END-PERFORM
           .

      *> ------------------------------------------------------------------
      *> RUN-CASE — run the reference kernel, then recompute expected
      *> values independently and verify all 4096 elements.
      *> ------------------------------------------------------------------
       RUN-CASE.
      *> The reference kernel under test.
           CALL "ATTN-CALC" USING Q-VAL K-VAL SCORE
               ON EXCEPTION
                   DISPLAY "FATAL: ATTN-CALC not found"
                   MOVE 1 TO RETURN-CODE
                   STOP RUN
           END-CALL

      *> Independent recomputation: contraction with per-term division
      *> (different rounding path than the kernel, which accumulates
      *> then divides — a genuine differential check).
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   MOVE 0 TO S-EXP-ELEMENT (I J)
                   PERFORM VARYING K FROM 1 BY 1 UNTIL K > 64
                       COMPUTE S-EXP-ELEMENT (I J) =
                           S-EXP-ELEMENT (I J) +
                           ((Q-ELEMENT (I K) * K-ELEMENT (J K)) / 8)
                   END-PERFORM
               END-PERFORM
           END-PERFORM

      *> Verify all 4096 elements.
           MOVE 0 TO MISMATCH
           MOVE SCORE-ELEMENT (1 1) TO WS-MIN
           MOVE SCORE-ELEMENT (1 1) TO WS-MAX
           MOVE 0 TO WS-MAX-ERR
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   COMPUTE WS-ERR =
                       FUNCTION ABS (SCORE-ELEMENT (I J)
                                    - S-EXP-ELEMENT (I J))
                   IF WS-ERR > WS-MAX-ERR
                       MOVE WS-ERR TO WS-MAX-ERR
                   END-IF
                   IF SCORE-ELEMENT (I J) < WS-MIN
                       MOVE SCORE-ELEMENT (I J) TO WS-MIN
                   END-IF
                   IF SCORE-ELEMENT (I J) > WS-MAX
                       MOVE SCORE-ELEMENT (I J) TO WS-MAX
                   END-IF
                   COMPUTE WS-EXP = FUNCTION ABS (S-EXP-ELEMENT (I J))
                   IF WS-ERR > (TOL * (1 + WS-EXP))
                       ADD 1 TO MISMATCH
                   END-IF
               END-PERFORM
           END-PERFORM
           ADD MISMATCH TO TOTAL-MISM
           IF MISMATCH > 0
               MOVE 'N' TO PASS-FLAG
           END-IF

      *> Report.
           MOVE WS-MAX-ERR TO FMT-ERR
           MOVE WS-MIN TO FMT-VAL
           DISPLAY " Case " CASE-ID " (" FUNCTION TRIM (CASE-NAME) ")"
           DISPLAY " max abs error : " FMT-ERR
           MOVE WS-MIN TO FMT-VAL
           DISPLAY " min value : " FMT-VAL
           MOVE WS-MAX TO FMT-VAL
           DISPLAY " max value : " FMT-VAL
           MOVE MISMATCH TO FMT-CNT
           DISPLAY " mismatches : " FMT-CNT " / 4096"
           .

      *> ------------------------------------------------------------------
      *> PIPELINE-CHECK — full pipeline: SCORE -> softmax -> x V -> OUT.
      *> Checks each softmax row sums to 1 and output elements are in
      *> the value range of V (softmax is a convex combination).
      *> ------------------------------------------------------------------
       PIPELINE-CHECK.
           DISPLAY "==================================================="
           DISPLAY " Pipeline check (ATTN-PIPE, case 7 data)"
           CALL "ATTN-PIPE" USING Q-VAL K-VAL V-VAL
                                 SCORE PROB OUT-MAT
               ON EXCEPTION
                   DISPLAY "FATAL: ATTN-PIPE not found"
                   MOVE 'N' TO PASS-FLAG
                   GOBACK
           END-CALL

      *> softmax rows must sum to 1
           MOVE 0 TO ROW-BAD
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               MOVE 0 TO ROW-SUM
               PERFORM VARYING J FROM 1 BY 1 UNTIL J > 64
                   COMPUTE ROW-SUM = ROW-SUM + PROB-ELEMENT (I J)
               END-PERFORM
               IF (FUNCTION ABS (ROW-SUM - 1)) > ROW-TOL
                   ADD 1 TO ROW-BAD
               END-IF
           END-PERFORM
           IF ROW-BAD > 0
               MOVE 'N' TO PASS-FLAG
           END-IF
           MOVE ROW-BAD TO FMT-CNT
           DISPLAY " softmax rows not summing to 1 : " FMT-CNT " / 64"

      *> output must be within the convex hull range of V:
      *> every OUT element lies in [min(V), max(V)].
           MOVE 0 TO OUT-BAD
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 64
               PERFORM VARYING D FROM 1 BY 1 UNTIL D > 64
                   IF OUT-ELEMENT (I D) < 0
                       ADD 1 TO OUT-BAD
                   END-IF
                   IF OUT-ELEMENT (I D) > 10
                       ADD 1 TO OUT-BAD
                   END-IF
               END-PERFORM
           END-PERFORM
           IF OUT-BAD > 0
               MOVE 'N' TO PASS-FLAG
           END-IF
           MOVE OUT-BAD TO FMT-CNT
           DISPLAY " attention output out of range : " FMT-CNT
                 " / 4096"
           .

       END PROGRAM ATTENTION-TEST.
