      ******************************************************************
      * BATCH-INGEST.CBL
      *
      * Batch ingestion of already-validated WORM ledger records.
      * Reads INGEST-FILE (records validated upstream by Ada/SPARK,
      * including all cryptographic checks) and appends each record to
      * the WORM ledger file (OPEN EXTEND only - never rewrites).
      *
      * This program performs a belt-and-suspenders STRUCTURAL check
      * only, NOT cryptographic re-verification:
      *   1. WR-SEQUENCE of the incoming record must equal
      *      (prior ledger sequence + 1).
      *   2. WR-PREV-HASH of the incoming record must string-equal
      *      WR-RECORD-HASH of the prior ledger record.
      *
      * Any record failing either check is routed to the exception file
      * (EXCEPT-FILE) instead of the ledger, along with a reason code.
      * Ingestion never halts on a single bad record; it fails that
      * record only and continues, so one rejected record does not
      * block the rest of the batch.  A summary is displayed at the end.
      ******************************************************************
       IDENTIFICATION DIVISION.
       PROGRAM-ID. BATCH-INGEST.
       AUTHOR. SOVEREIGN-ENGINE-V2.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. GENERIC.
       OBJECT-COMPUTER. GENERIC.

       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT INGEST-FILE ASSIGN TO "INGEST"
               ORGANIZATION IS LINE SEQUENTIAL
               FILE STATUS IS WS-INGEST-STATUS.

           SELECT WORM-LEDGER-FILE ASSIGN TO "WORMLDGR"
               ORGANIZATION IS LINE SEQUENTIAL
               FILE STATUS IS WS-LEDGER-STATUS.

           SELECT LEDGER-READ-FILE ASSIGN TO "WORMLDGR"
               ORGANIZATION IS LINE SEQUENTIAL
               FILE STATUS IS WS-LREAD-STATUS.

           SELECT EXCEPT-FILE ASSIGN TO "EXCEPT"
               ORGANIZATION IS LINE SEQUENTIAL
               FILE STATUS IS WS-EXCEPT-STATUS.

       DATA DIVISION.
       FILE SECTION.
       FD  INGEST-FILE
           RECORDING MODE IS F.
       01  INGEST-RECORD-LINE.
           COPY WORM-RECORD.

       FD  WORM-LEDGER-FILE
           RECORDING MODE IS F.
       01  LEDGER-WRITE-LINE.
           COPY WORM-RECORD.

       FD  LEDGER-READ-FILE
           RECORDING MODE IS F.
       01  LEDGER-READ-LINE.
           COPY WORM-RECORD.

       FD  EXCEPT-FILE
           RECORDING MODE IS F.
       01  EXCEPT-RECORD-LINE.
           05  EX-REASON-CODE         PIC X(08).
           05  EX-REASON-TEXT         PIC X(60).
           05  EX-ORIGINAL-RECORD.
               COPY WORM-RECORD.

       WORKING-STORAGE SECTION.
       01  WS-INGEST-STATUS           PIC X(02) VALUE SPACES.
           88  WS-INGEST-OK           VALUE '00'.

       01  WS-LEDGER-STATUS           PIC X(02) VALUE SPACES.
           88  WS-LEDGER-OK           VALUE '00'.

       01  WS-LREAD-STATUS            PIC X(02) VALUE SPACES.
           88  WS-LREAD-OK            VALUE '00'.
           88  WS-LREAD-EOF           VALUE '10'.

       01  WS-EXCEPT-STATUS           PIC X(02) VALUE SPACES.
           88  WS-EXCEPT-OK           VALUE '00'.

       01  WS-SWITCHES.
           05  WS-INGEST-EOF-SW       PIC X(01) VALUE 'N'.
               88  WS-INGEST-EOF      VALUE 'Y'.
           05  WS-LEDGER-EMPTY-SW     PIC X(01) VALUE 'Y'.
               88  WS-LEDGER-EMPTY    VALUE 'Y'.
           05  WS-RECORD-OK-SW        PIC X(01) VALUE 'Y'.
               88  WS-RECORD-OK       VALUE 'Y'.
               88  WS-RECORD-BAD      VALUE 'N'.

       01  WS-PRIOR-STATE.
           05  WS-PRIOR-SEQUENCE      PIC 9(18) VALUE ZERO.
           05  WS-PRIOR-RECORD-HASH   PIC X(64) VALUE SPACES.

       01  WS-EXPECTED-SEQUENCE       PIC 9(18) VALUE ZERO.

       01  WS-COUNTERS.
           05  WS-READ-COUNT          PIC 9(09) VALUE ZERO.
           05  WS-APPEND-COUNT        PIC 9(09) VALUE ZERO.
           05  WS-REJECT-COUNT        PIC 9(09) VALUE ZERO.

       01  WS-REJECT-PARMS.
           05  WS-REJECT-CODE         PIC X(08) VALUE SPACES.
           05  WS-REJECT-TEXT         PIC X(60) VALUE SPACES.

       PROCEDURE DIVISION.
       0000-MAIN.
           PERFORM 1000-INITIALIZE
           PERFORM 2000-INGEST-LOOP
               UNTIL WS-INGEST-EOF
           PERFORM 9000-TERMINATE
           STOP RUN.

       1000-INITIALIZE.
           PERFORM 1100-DETERMINE-PRIOR-STATE

           OPEN INPUT INGEST-FILE
           IF NOT WS-INGEST-OK
               DISPLAY "BATCH-INGEST: cannot open INGEST-FILE, status "
                   WS-INGEST-STATUS
               STOP RUN
           END-IF

           OPEN EXTEND WORM-LEDGER-FILE
           IF NOT WS-LEDGER-OK
               DISPLAY "BATCH-INGEST: cannot open ledger EXTEND, status "
                   WS-LEDGER-STATUS
               CLOSE INGEST-FILE
               STOP RUN
           END-IF

           OPEN OUTPUT EXCEPT-FILE
           IF NOT WS-EXCEPT-OK
               DISPLAY "BATCH-INGEST: cannot open EXCEPT-FILE, status "
                   WS-EXCEPT-STATUS
               CLOSE INGEST-FILE
               CLOSE WORM-LEDGER-FILE
               STOP RUN
           END-IF

           PERFORM 8100-READ-NEXT-INGEST.

      * Read the existing ledger sequentially to find the last record's
      * sequence number and record-hash, which anchor the belt-and-
      * suspenders checks for the first incoming record of this batch.
       1100-DETERMINE-PRIOR-STATE.
           OPEN INPUT LEDGER-READ-FILE
           IF NOT WS-LREAD-OK
      * Ledger file does not exist yet or cannot be opened - treat as
      * an empty ledger; first incoming record must have sequence 1
      * and is exempt from the prev-hash check.
               MOVE ZERO TO WS-PRIOR-SEQUENCE
               MOVE SPACES TO WS-PRIOR-RECORD-HASH
               SET WS-LEDGER-EMPTY TO TRUE
           ELSE
               PERFORM UNTIL WS-LREAD-EOF
                   READ LEDGER-READ-FILE
                       AT END
                           CONTINUE
                       NOT AT END
                           MOVE WR-SEQUENCE OF LEDGER-READ-LINE
                               TO WS-PRIOR-SEQUENCE
                           MOVE WR-RECORD-HASH OF LEDGER-READ-LINE
                               TO WS-PRIOR-RECORD-HASH
                           MOVE 'N' TO WS-LEDGER-EMPTY-SW
                   END-READ
               END-PERFORM
               CLOSE LEDGER-READ-FILE
           END-IF.

       2000-INGEST-LOOP.
           ADD 1 TO WS-READ-COUNT
           SET WS-RECORD-OK TO TRUE
           COMPUTE WS-EXPECTED-SEQUENCE = WS-PRIOR-SEQUENCE + 1

           IF WR-SEQUENCE OF INGEST-RECORD-LINE
                   NOT = WS-EXPECTED-SEQUENCE
               SET WS-RECORD-BAD TO TRUE
               MOVE "SEQGAP  " TO WS-REJECT-CODE
               MOVE "sequence not prior+1" TO WS-REJECT-TEXT
               PERFORM 3000-ROUTE-TO-EXCEPTION
           ELSE
               IF NOT WS-LEDGER-EMPTY
                   IF WR-PREV-HASH OF INGEST-RECORD-LINE
                           NOT = WS-PRIOR-RECORD-HASH
                       SET WS-RECORD-BAD TO TRUE
                       MOVE "HASHMIS " TO WS-REJECT-CODE
                       MOVE "prev-hash mismatch" TO WS-REJECT-TEXT
                       PERFORM 3000-ROUTE-TO-EXCEPTION
                   END-IF
               END-IF
           END-IF

           IF WS-RECORD-OK
               MOVE INGEST-RECORD-LINE TO LEDGER-WRITE-LINE
               WRITE LEDGER-WRITE-LINE
               IF WS-LEDGER-OK
                   ADD 1 TO WS-APPEND-COUNT
                   MOVE WR-SEQUENCE OF INGEST-RECORD-LINE
                       TO WS-PRIOR-SEQUENCE
                   MOVE WR-RECORD-HASH OF INGEST-RECORD-LINE
                       TO WS-PRIOR-RECORD-HASH
                   MOVE 'N' TO WS-LEDGER-EMPTY-SW
               ELSE
                   DISPLAY "BATCH-INGEST: ledger WRITE failed, status "
                       WS-LEDGER-STATUS
                   MOVE "WRTFAIL " TO WS-REJECT-CODE
                   MOVE "ledger write failed" TO WS-REJECT-TEXT
                   PERFORM 3000-ROUTE-TO-EXCEPTION
               END-IF
           END-IF

           PERFORM 8100-READ-NEXT-INGEST.

       3000-ROUTE-TO-EXCEPTION.
           MOVE SPACES TO EXCEPT-RECORD-LINE
           MOVE WS-REJECT-CODE TO EX-REASON-CODE
           MOVE WS-REJECT-TEXT TO EX-REASON-TEXT
           MOVE INGEST-RECORD-LINE TO EX-ORIGINAL-RECORD
           WRITE EXCEPT-RECORD-LINE
           ADD 1 TO WS-REJECT-COUNT
           DISPLAY "BATCH-INGEST: rejected seq="
               WR-SEQUENCE OF INGEST-RECORD-LINE
               " record-id=" WR-RECORD-ID OF INGEST-RECORD-LINE
               " reason=" WS-REJECT-TEXT.

       8100-READ-NEXT-INGEST.
           READ INGEST-FILE
               AT END
                   SET WS-INGEST-EOF TO TRUE
           END-READ.

       9000-TERMINATE.
           CLOSE INGEST-FILE
           CLOSE WORM-LEDGER-FILE
           CLOSE EXCEPT-FILE
           DISPLAY "BATCH-INGEST: read=" WS-READ-COUNT
               " appended=" WS-APPEND-COUNT
               " rejected=" WS-REJECT-COUNT.
