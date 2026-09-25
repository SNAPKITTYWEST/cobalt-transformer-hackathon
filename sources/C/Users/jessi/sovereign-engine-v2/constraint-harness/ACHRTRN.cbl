       IDENTIFICATION DIVISION.
       PROGRAM-ID. ACHMATRIX.
       AUTHOR.     JESSICA+COPILOT.
      *================================================================
      * ACH RETURN MATRIX PROCESSOR
      * State transition via matrix composition
      * No functor - pure array operations
      *================================================================
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. IBM-Z.
       OBJECT-COMPUTER. IBM-Z.

       DATA DIVISION.

       WORKING-STORAGE SECTION.

      *================================================================
      * STATE VECTOR ENCODING
      * 0=NEW 1=POSTED 2=SETTLED 3=RETURNED 4=RETURN_POSTED 5=CLOSED
      *================================================================
       01  STATE-VECTOR.
           05 STATE-DIMENSION         PIC 9(02) VALUE 6.
           05 STATE-CURRENT           PIC 9(02) VALUE 0.
           05 STATE-TARGET            PIC 9(02) VALUE 0.
           05 STATE-ENCODING OCCURS 6 TIMES.
              10 STATE-NAME           PIC X(12).
              10 STATE-INDEX          PIC 9(02).

      *================================================================
      * STATE TRANSITION MATRIX (6x6)
      * M[i][j] = 1 if transition from state i to state j is valid
      *================================================================
       01  TRANSITION-MATRIX.
           05 MATRIX-ROW OCCURS 6 TIMES.
              10 MATRIX-COL OCCURS 6 TIMES PIC 9(01).

      *================================================================
      * REASON CODE VALIDATION MATRIX (20x1)
      *================================================================
       01  REASON-MATRIX.
           05 REASON-VECTOR OCCURS 20 TIMES.
              10 REASON-CODE          PIC X(03).
              10 REASON-VALID         PIC 9(01).

      *================================================================
      * INPUT/OUTPUT STATE ARRAYS
      *================================================================
       01  INPUT-ARRAY.
           05 IN-STATE-INDEX          PIC 9(02).
           05 IN-REASON-INDEX         PIC 9(02).
           05 IN-COMPANY              PIC X(03).
           05 IN-BATCH-ID             PIC X(10).
           05 IN-ENTRY-ID             PIC X(15).

       01  OUTPUT-ARRAY.
           05 OUT-STATE-INDEX         PIC 9(02).
           05 OUT-TRANSITION-VALID    PIC 9(01).
           05 OUT-ERROR-CODE          PIC X(08).
           05 OUT-ERROR-MSG           PIC X(80).

      *================================================================
      * MATRIX COMPUTATION WORKSPACE
      *================================================================
       01  MATRIX-WORKSPACE.
           05 WS-ROW-IDX              PIC 9(02).
           05 WS-COL-IDX              PIC 9(02).
           05 WS-TEMP-SUM             PIC 9(04).
           05 WS-MATRIX-PRODUCT       PIC 9(01).

       LINKAGE SECTION.
       01  LK-INPUT-ARRAY.
           05 LK-IN-STATE-INDEX       PIC 9(02).
           05 LK-IN-REASON-INDEX      PIC 9(02).
           05 LK-IN-COMPANY           PIC X(03).
           05 LK-IN-BATCH-ID          PIC X(10).
           05 LK-IN-ENTRY-ID          PIC X(15).

       01  LK-OUTPUT-ARRAY.
           05 LK-OUT-STATE-INDEX      PIC 9(02).
           05 LK-OUT-TRANSITION-VALID PIC 9(01).
           05 LK-OUT-ERROR-CODE       PIC X(08).
           05 LK-OUT-ERROR-MSG        PIC X(80).

       PROCEDURE DIVISION USING LK-INPUT-ARRAY LK-OUTPUT-ARRAY.

       MAIN-MATRIX-PROCESSOR.
           PERFORM INIT-STATE-ENCODING
           PERFORM INIT-TRANSITION-MATRIX
           PERFORM INIT-REASON-MATRIX
           PERFORM LOAD-INPUT-VECTOR
           PERFORM COMPUTE-STATE-TRANSITION
           PERFORM STORE-OUTPUT-VECTOR
           GOBACK.

      *================================================================
      * INIT-STATE-ENCODING: Map state names to indices
      *================================================================
       INIT-STATE-ENCODING.
           MOVE 'NEW'           TO STATE-NAME(1)
           MOVE 0               TO STATE-INDEX(1)
           MOVE 'POSTED'        TO STATE-NAME(2)
           MOVE 1               TO STATE-INDEX(2)
           MOVE 'SETTLED'       TO STATE-NAME(3)
           MOVE 2               TO STATE-INDEX(3)
           MOVE 'RETURNED'      TO STATE-NAME(4)
           MOVE 3               TO STATE-INDEX(4)
           MOVE 'RETURN_POSTED' TO STATE-NAME(5)
           MOVE 4               TO STATE-INDEX(5)
           MOVE 'CLOSED'        TO STATE-NAME(6)
           MOVE 5               TO STATE-INDEX(6).

      *================================================================
      * INIT-TRANSITION-MATRIX: Define valid state transitions
      * Matrix algebra: M[i][j]=1 means i→j is valid
      *================================================================
       INIT-TRANSITION-MATRIX.
      *    Initialize all to 0 (no transition)
           PERFORM VARYING WS-ROW-IDX FROM 1 BY 1 UNTIL WS-ROW-IDX > 6
              PERFORM VARYING WS-COL-IDX FROM 1 BY 1 UNTIL WS-COL-IDX > 6
                 MOVE 0 TO MATRIX-COL(WS-ROW-IDX, WS-COL-IDX)
              END-PERFORM
           END-PERFORM

      *    Define valid transitions
      *    POSTED (idx=2) -> RETURNED (idx=4)
           MOVE 1 TO MATRIX-COL(2, 4)
      *    SETTLED (idx=3) -> RETURNED (idx=4)
           MOVE 1 TO MATRIX-COL(3, 4)
      *    Self-loops for idempotency check
           MOVE 1 TO MATRIX-COL(1, 1)
           MOVE 1 TO MATRIX-COL(2, 2)
           MOVE 1 TO MATRIX-COL(3, 3)
           MOVE 1 TO MATRIX-COL(4, 4)
           MOVE 1 TO MATRIX-COL(5, 5)
           MOVE 1 TO MATRIX-COL(6, 6).

      *================================================================
      * INIT-REASON-MATRIX: Encode NACHA reason codes
      *================================================================
       INIT-REASON-MATRIX.
           PERFORM VARYING WS-ROW-IDX FROM 1 BY 1 UNTIL WS-ROW-IDX > 20
              MOVE SPACES TO REASON-CODE(WS-ROW-IDX)
              MOVE 0 TO REASON-VALID(WS-ROW-IDX)
           END-PERFORM

           MOVE 'R01' TO REASON-CODE(1)
           MOVE 1 TO REASON-VALID(1)
           MOVE 'R03' TO REASON-CODE(2)
           MOVE 1 TO REASON-VALID(2)
           MOVE 'R04' TO REASON-CODE(3)
           MOVE 1 TO REASON-VALID(3)
           MOVE 'R07' TO REASON-CODE(4)
           MOVE 1 TO REASON-VALID(4)
           MOVE 'R08' TO REASON-CODE(5)
           MOVE 1 TO REASON-VALID(5)
           MOVE 'R10' TO REASON-CODE(6)
           MOVE 1 TO REASON-VALID(6)
           MOVE 'R29' TO REASON-CODE(7)
           MOVE 1 TO REASON-VALID(7).

      *================================================================
      * LOAD-INPUT-VECTOR: Map linkage to internal arrays
      *================================================================
       LOAD-INPUT-VECTOR.
           MOVE LK-IN-STATE-INDEX  TO IN-STATE-INDEX
           MOVE LK-IN-REASON-INDEX TO IN-REASON-INDEX
           MOVE LK-IN-COMPANY      TO IN-COMPANY
           MOVE LK-IN-BATCH-ID     TO IN-BATCH-ID
           MOVE LK-IN-ENTRY-ID     TO IN-ENTRY-ID.

      *================================================================
      * COMPUTE-STATE-TRANSITION: Pure matrix algebra
      * Check M[current_state][RETURNED_STATE]
      *================================================================
       COMPUTE-STATE-TRANSITION.
           MOVE IN-STATE-INDEX TO STATE-CURRENT
           MOVE 4 TO STATE-TARGET

      *    Matrix lookup: can we transition from current to RETURNED?
           IF STATE-CURRENT < 1 OR STATE-CURRENT > 6
              MOVE 0 TO OUT-TRANSITION-VALID
              MOVE 'BADSTATE' TO OUT-ERROR-CODE
              MOVE 'Invalid source state index' TO OUT-ERROR-MSG
              EXIT PARAGRAPH
           END-IF

      *    Check transition matrix
           IF MATRIX-COL(STATE-CURRENT, STATE-TARGET) = 1
              MOVE 1 TO OUT-TRANSITION-VALID
              MOVE STATE-TARGET TO OUT-STATE-INDEX
              MOVE 'OK' TO OUT-ERROR-CODE
              MOVE SPACES TO OUT-ERROR-MSG
           ELSE
              MOVE 0 TO OUT-TRANSITION-VALID
              MOVE 'NOTALLOW' TO OUT-ERROR-CODE
              MOVE 'Transition not allowed by matrix' TO OUT-ERROR-MSG
           END-IF

      *    Validate reason code via vector lookup
           IF OUT-TRANSITION-VALID = 1
              IF IN-REASON-INDEX > 0 AND IN-REASON-INDEX <= 20
                 IF REASON-VALID(IN-REASON-INDEX) = 0
                    MOVE 0 TO OUT-TRANSITION-VALID
                    MOVE 'BADREASN' TO OUT-ERROR-CODE
                    MOVE 'Invalid reason code' TO OUT-ERROR-MSG
                 END-IF
              ELSE
                 MOVE 0 TO OUT-TRANSITION-VALID
                 MOVE 'BADREASN' TO OUT-ERROR-CODE
                 MOVE 'Reason code out of range' TO OUT-ERROR-MSG
              END-IF
           END-IF.

      *================================================================
      * STORE-OUTPUT-VECTOR: Write results to linkage
      *================================================================
       STORE-OUTPUT-VECTOR.
           MOVE OUT-STATE-INDEX      TO LK-OUT-STATE-INDEX
           MOVE OUT-TRANSITION-VALID TO LK-OUT-TRANSITION-VALID
           MOVE OUT-ERROR-CODE       TO LK-OUT-ERROR-CODE
           MOVE OUT-ERROR-MSG        TO LK-OUT-ERROR-MSG.

       WORKING-STORAGE SECTION.

       01  WS-PROGRAM-NAME            PIC X(08) VALUE 'ACHRTRN'.
       01  WS-RUN-MODE                PIC X(01) VALUE 'B'. *> B=batch, O=online

       01  AI-STATUS                  PIC X(02) VALUE SPACES.
       01  AR-STATUS                  PIC X(02) VALUE SPACES.

       01  WS-RETURN-REQUEST.
           05 WR-COMPANY              PIC X(03).
           05 WR-BATCH-ID             PIC X(10).
           05 WR-ENTRY-ID             PIC X(15).
           05 WR-USER-ID              PIC X(10).
           05 WR-CHANNEL              PIC X(08).
           05 WR-REASON-CODE          PIC X(03).

       01  WS-RETURN-RESPONSE.
           05 WRS-SUCCESS             PIC X(01). *> 'Y' or 'N'
           05 WRS-ERROR-CODE          PIC X(08).
           05 WRS-ERROR-MSG           PIC X(80).
           05 WRS-LEDGER-SEQ          PIC 9(09).

       01  WS-TIMESTAMP               PIC X(26).
       01  WS-LOG-SEQ                 PIC 9(09) VALUE 0.

       01  WS-STATE-TARGET            PIC X(12).

       01  WS-REASON-VALID            PIC X(01) VALUE 'N'.

       01  WS-RETURNABLE-STATE        PIC X(12).

       01  WS-ABEND-FLAG              PIC X(01) VALUE 'N'.

       01  WS-DISPLAY-MSG             PIC X(80).

       01  FILLER REDEFINES WS-TIMESTAMP.
           05 WS-TS-YYYY              PIC X(04).
           05 WS-TS-MM                PIC X(02).
           05 WS-TS-DD                PIC X(02).
           05 WS-TS-T                 PIC X(01).
           05 WS-TS-HH                PIC X(02).
           05 WS-TS-MI                PIC X(02).
           05 WS-TS-SS                PIC X(02).
           05 WS-TS-DOT               PIC X(01).
           05 WS-TS-MSEC              PIC X(03).
           05 WS-TS-Z                 PIC X(01).
           05 WS-TS-OFFSET            PIC X(06).

       01  WS-REASON-TABLE.
           05 WS-REASON-ENTRY OCCURS 20 TIMES INDEXED BY REASON-IDX.
              10 WS-REASON-CODE       PIC X(03).
              10 WS-REASON-DESC       PIC X(40).

       01  WS-INIT-REASONS-SW         PIC X(01) VALUE 'N'.

       LINKAGE SECTION.
       01  LK-RETURN-REQUEST.
           05 LK-COMPANY              PIC X(03).
           05 LK-BATCH-ID             PIC X(10).
           05 LK-ENTRY-ID             PIC X(15).
           05 LK-USER-ID              PIC X(10).
           05 LK-CHANNEL              PIC X(08).
           05 LK-REASON-CODE          PIC X(03).

       01  LK-RETURN-RESPONSE.
           05 LK-SUCCESS              PIC X(01).
           05 LK-ERROR-CODE           PIC X(08).
           05 LK-ERROR-MSG            PIC X(80).
           05 LK-LEDGER-SEQ           PIC 9(09).

       PROCEDURE DIVISION USING LK-RETURN-REQUEST LK-RETURN-RESPONSE.

       MAIN-SECTION.
           PERFORM INIT-SECTION
           PERFORM LOAD-REQUEST
           PERFORM PROCESS-RETURN
           PERFORM BUILD-RESPONSE
           GOBACK.

       INIT-SECTION.
           IF WS-INIT-REASONS-SW = 'N'
              PERFORM INIT-REASON-TABLE
              MOVE 'Y' TO WS-INIT-REASONS-SW
           END-IF

           MOVE 'N' TO WRS-SUCCESS
           MOVE SPACES TO WRS-ERROR-CODE WRS-ERROR-MSG
           MOVE ZEROES TO WRS-LEDGER-SEQ

           OPEN I-O ACHITEM-FILE
           IF AI-STATUS NOT = '00'
              MOVE 'Y' TO WS-ABEND-FLAG
              MOVE 'FILEOPEN' TO WRS-ERROR-CODE
              MOVE 'ACHITEM open failed' TO WRS-ERROR-MSG
              PERFORM ABEND-SECTION
           END-IF

           OPEN I-O ACHRETLOG-FILE
           IF AR-STATUS NOT = '00'
              MOVE 'Y' TO WS-ABEND-FLAG
              MOVE 'FILEOPEN' TO WRS-ERROR-CODE
              MOVE 'ACHRETLOG open failed' TO WRS-ERROR-MSG
              PERFORM ABEND-SECTION
           END-IF

           PERFORM GET-CURRENT-TIMESTAMP.

       LOAD-REQUEST.
           MOVE LK-COMPANY     TO WR-COMPANY
           MOVE LK-BATCH-ID    TO WR-BATCH-ID
           MOVE LK-ENTRY-ID    TO WR-ENTRY-ID
           MOVE LK-USER-ID     TO WR-USER-ID
           MOVE LK-CHANNEL     TO WR-CHANNEL
           MOVE LK-REASON-CODE TO WR-REASON-CODE.

       PROCESS-RETURN.
           PERFORM LOAD-ACH-ITEM
           IF WRS-ERROR-CODE NOT = SPACES
              PERFORM LOG-RETURN-FAIL
              EXIT PARAGRAPH
           END-IF

           PERFORM VALIDATE-RETURN-ELIGIBILITY
           IF WRS-ERROR-CODE NOT = SPACES
              PERFORM LOG-RETURN-FAIL
              EXIT PARAGRAPH
           END-IF

           PERFORM APPLY-STATE-TRANSITION
           IF WRS-ERROR-CODE NOT = SPACES
              PERFORM LOG-RETURN-FAIL
              EXIT PARAGRAPH
           END-IF

           PERFORM PERSIST-ACH-ITEM
           IF WRS-ERROR-CODE NOT = SPACES
              PERFORM LOG-RETURN-FAIL
              EXIT PARAGRAPH
           END-IF

           PERFORM LOG-RETURN-SUCCESS.

       BUILD-RESPONSE.
           MOVE WRS-SUCCESS    TO LK-SUCCESS
           MOVE WRS-ERROR-CODE TO LK-ERROR-CODE
           MOVE WRS-ERROR-MSG  TO LK-ERROR-MSG
           MOVE WRS-LEDGER-SEQ TO LK-LEDGER-SEQ.

       LOAD-ACH-ITEM.
           MOVE WR-COMPANY  TO AI-COMPANY
           MOVE WR-BATCH-ID TO AI-BATCH-ID
           MOVE WR-ENTRY-ID TO AI-ENTRY-ID

           READ ACHITEM-FILE
               INVALID KEY
                   MOVE 'NOTFOUND' TO WRS-ERROR-CODE
                   MOVE 'ACH item not found' TO WRS-ERROR-MSG
               NOT INVALID KEY
                   CONTINUE
           END-READ.

       VALIDATE-RETURN-ELIGIBILITY.
           IF WRS-ERROR-CODE NOT = SPACES
              EXIT PARAGRAPH
           END-IF

           *> Only SETTLED or POSTED items can be returned
           IF AI-STATE = 'SETTLED'
              MOVE 'SETTLED' TO WS-RETURNABLE-STATE
           ELSE
              IF AI-STATE = 'POSTED'
                 MOVE 'POSTED' TO WS-RETURNABLE-STATE
              ELSE
                 MOVE 'BADSTATE' TO WRS-ERROR-CODE
                 MOVE 'Item state not returnable: ' TO WRS-ERROR-MSG
                 STRING AI-STATE DELIMITED BY SIZE
                        INTO WRS-ERROR-MSG
                 EXIT PARAGRAPH
              END-IF
           END-IF

           *> Prevent double return
           IF AI-STATE = 'RETURNED'
              MOVE 'ALREADYRT' TO WRS-ERROR-CODE
              MOVE 'Item already returned' TO WRS-ERROR-MSG
              EXIT PARAGRAPH
           END-IF

           *> Validate reason code
           PERFORM CHECK-REASON-CODE
           IF WS-REASON-VALID NOT = 'Y'
              MOVE 'BADREASN' TO WRS-ERROR-CODE
              MOVE 'Invalid return reason code' TO WRS-ERROR-MSG
              EXIT PARAGRAPH
           END-IF.

       APPLY-STATE-TRANSITION.
           IF WRS-ERROR-CODE NOT = SPACES
              EXIT PARAGRAPH
           END-IF

           *> State machine:
           *> SETTLED -> RETURNED
           *> POSTED  -> RETURNED
           MOVE 'RETURNED' TO WS-STATE-TARGET

           MOVE WS-STATE-TARGET TO AI-STATE
           MOVE WR-REASON-CODE  TO AI-RETURN-REASON
           MOVE WR-USER-ID      TO AI-RETURN-USER
           MOVE WR-CHANNEL      TO AI-RETURN-CHANNEL
           MOVE WS-TIMESTAMP    TO AI-RETURN-TS
           MOVE WS-TIMESTAMP    TO AI-LAST-UPD-TS.

           *> Ledger seq will be filled by downstream posting engine
           MOVE ZEROES TO AI-RETURN-LEDGER-SEQ.

       PERSIST-ACH-ITEM.
           REWRITE ACHITEM-REC
               INVALID KEY
                   MOVE 'DBERR' TO WRS-ERROR-CODE
                   MOVE 'ACH item rewrite failed' TO WRS-ERROR-MSG
               NOT INVALID KEY
                   CONTINUE
           END-REWRITE.

       LOG-RETURN-FAIL.
           PERFORM NEXT-LOG-SEQ
           MOVE WR-COMPANY   TO AR-COMPANY
           MOVE WR-BATCH-ID  TO AR-BATCH-ID
           MOVE WR-ENTRY-ID  TO AR-ENTRY-ID
           MOVE WS-LOG-SEQ   TO AR-LOG-SEQ
           MOVE 'RET_FAIL'   TO AR-EVENT-CODE
           MOVE WS-TIMESTAMP TO AR-EVENT-TS
           MOVE WR-USER-ID   TO AR-USER-ID
           MOVE WR-CHANNEL   TO AR-CHANNEL
           MOVE WR-REASON-CODE TO AR-REASON-CODE
           MOVE SPACES       TO AR-DETAIL

           STRING 'Return failed: '
                  WRS-ERROR-CODE DELIMITED BY SIZE
                  ' - ' DELIMITED BY SIZE
                  WRS-ERROR-MSG DELIMITED BY SIZE
                  INTO AR-DETAIL
           END-STRING

           WRITE ACHRETLOG-REC
               INVALID KEY
                   CONTINUE
           END-WRITE.

       LOG-RETURN-SUCCESS.
           MOVE 'Y' TO WRS-SUCCESS
           MOVE 'OK' TO WRS-ERROR-CODE
           MOVE 'Return accepted' TO WRS-ERROR-MSG

           PERFORM NEXT-LOG-SEQ
           MOVE WR-COMPANY   TO AR-COMPANY
           MOVE WR-BATCH-ID  TO AR-BATCH-ID
           MOVE WR-ENTRY-ID  TO AR-ENTRY-ID
           MOVE WS-LOG-SEQ   TO AR-LOG-SEQ
           MOVE 'RET_REQ'    TO AR-EVENT-CODE
           MOVE WS-TIMESTAMP TO AR-EVENT-TS
           MOVE WR-USER-ID   TO AR-USER-ID
           MOVE WR-CHANNEL   TO AR-CHANNEL
           MOVE WR-REASON-CODE TO AR-REASON-CODE
           MOVE SPACES       TO AR-DETAIL

           STRING 'Return requested; state='
                  AI-STATE DELIMITED BY SIZE
                  ' reason=' DELIMITED BY SIZE
                  WR-REASON-CODE DELIMITED BY SIZE
                  INTO AR-DETAIL
           END-STRING

           WRITE ACHRETLOG-REC
               INVALID KEY
                   CONTINUE
           END-WRITE.

       CHECK-REASON-CODE.
           MOVE 'N' TO WS-REASON-VALID
           SET REASON-IDX TO 1
           PERFORM VARYING REASON-IDX FROM 1 BY 1
                   UNTIL REASON-IDX > 20
              IF WS-REASON-CODE (REASON-IDX) = WR-REASON-CODE
                 MOVE 'Y' TO WS-REASON-VALID
                 EXIT PERFORM
              END-IF
           END-PERFORM.

       INIT-REASON-TABLE.
           MOVE 'R01' TO WS-REASON-CODE (1)
           MOVE 'Insufficient funds' TO WS-REASON-DESC (1)

           MOVE 'R03' TO WS-REASON-CODE (2)
           MOVE 'No account/Unable to locate' TO WS-REASON-DESC (2)

           MOVE 'R04' TO WS-REASON-CODE (3)
           MOVE 'Invalid account number' TO WS-REASON-DESC (3)

           MOVE 'R07' TO WS-REASON-CODE (4)
           MOVE 'Authorization revoked' TO WS-REASON-DESC (4)

           MOVE 'R08' TO WS-REASON-CODE (5)
           MOVE 'Payment stopped' TO WS-REASON-DESC (5)

           MOVE 'R10' TO WS-REASON-CODE (6)
           MOVE 'Customer advises not authorized' TO WS-REASON-DESC (6)

           MOVE 'R29' TO WS-REASON-CODE (7)
           MOVE 'Corporate customer advises not authorized' TO WS-REASON-DESC (7)

           *> Remaining entries left blank; extend as needed.

       NEXT-LOG-SEQ.
           ADD 1 TO WS-LOG-SEQ.

       GET-CURRENT-TIMESTAMP.
           *> Stub: in production, call system service or LE routine
           MOVE '20260908T192700.000Z+0000' TO WS-TIMESTAMP.

       ABEND-SECTION.
           IF WS-ABEND-FLAG = 'Y'
              DISPLAY 'ACHRTRN ABEND: ' WRS-ERROR-CODE ' ' WRS-ERROR-MSG
              GOBACK
           END-IF.
