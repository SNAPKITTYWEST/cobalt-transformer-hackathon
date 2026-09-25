      ******************************************************************
      * RECONCILE-LEDGER.CBL
      *
      * Reads the WORM ledger file sequentially and verifies structural
      * continuity of the chain.  This is NOT cryptographic
      * re-verification: all hash comparisons here are plain string
      * equality against values already computed upstream by Ada/SPARK.
      *
      * Checks performed, per record and cumulatively:
      *   1. No sequence gaps      - WR-SEQUENCE must be prior+1.
      *   2. No duplicate sequence - WR-SEQUENCE must not repeat.
      *   3. No duplicate record-id- WR-RECORD-ID must not repeat.
      *   4. Prev-hash chain link  - WR-PREV-HASH must string-equal the
      *                              prior record's WR-RECORD-HASH.
      *
      * This program FAILS CLOSED: any anomaly on any record sets the
      * overall run status to FAIL and that fact is never overwritten
      * back to PASS.  Processing continues after an anomaly only so a
      * complete report can be produced; the final verdict is FAIL if
      * even one anomaly was found anywhere in the ledger.
      *
      * A duplicate-detection design note: this program assumes the
      * ledger fits within available working storage as an in-memory
      * table for duplicate sequence/record-id detection (see
      * WS-SEEN-TABLE).  For very large ledgers a sort-based or indexed
      * approach would be required instead; table size here is a fixed
      * upper bound (see WS-MAX-RECORDS) appropriate for a bounded
      * ledger segment, not unbounded production scale.
      ******************************************************************
       IDENTIFICATION DIVISION.
       PROGRAM-ID. RECONCILE-LEDGER.
       AUTHOR. SOVEREIGN-ENGINE-V2.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. GENERIC.
       OBJECT-COMPUTER. GENERIC.

       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT WORM-LEDGER-FILE ASSIGN TO "WORMLDGR"
               ORGANIZATION IS LINE SEQUENTIAL
               FILE STATUS IS WS-LEDGER-STATUS.

           SELECT REPORT-FILE ASSIGN TO "RECONRPT"
               ORGANIZATION IS LINE SEQUENTIAL
               FILE STATUS IS WS-REPORT-STATUS.

       DATA DIVISION.
       FILE SECTION.
       FD  WORM-LEDGER-FILE
           RECORDING MODE IS F.
       01  LEDGER-RECORD-LINE.
           COPY WORM-RECORD.

       FD  REPORT-FILE
           RECORDING MODE IS F.
       01  REPORT-LINE                PIC X(132).

       WORKING-STORAGE SECTION.
       01  WS-LEDGER-STATUS           PIC X(02) VALUE SPACES.
           88  WS-LEDGER-OK           VALUE '00'.

       01  WS-REPORT-STATUS           PIC X(02) VALUE SPACES.
           88  WS-REPORT-OK           VALUE '00'.

       01  WS-SWITCHES.
           05  WS-EOF-SW              PIC X(01) VALUE 'N'.
               88  WS-END-OF-LEDGER   VALUE 'Y'.
           05  WS-FIRST-RECORD-SW     PIC X(01) VALUE 'Y'.
               88  WS-IS-FIRST-RECORD VALUE 'Y'.
           05  WS-OVERALL-RESULT-SW   PIC X(04) VALUE 'PASS'.
               88  WS-OVERALL-PASS    VALUE 'PASS'.
               88  WS-OVERALL-FAIL    VALUE 'FAIL'.

       01  WS-PRIOR-STATE.
           05  WS-PRIOR-SEQUENCE      PIC 9(18) VALUE ZERO.
           05  WS-PRIOR-RECORD-HASH   PIC X(64) VALUE SPACES.

       01  WS-EXPECTED-SEQUENCE       PIC 9(18) VALUE ZERO.

       01  WS-COUNTERS.
           05  WS-RECORD-COUNT        PIC 9(09) VALUE ZERO.
           05  WS-ANOMALY-COUNT       PIC 9(09) VALUE ZERO.
           05  WS-SEQGAP-COUNT        PIC 9(09) VALUE ZERO.
           05  WS-DUPSEQ-COUNT        PIC 9(09) VALUE ZERO.
           05  WS-DUPID-COUNT         PIC 9(09) VALUE ZERO.
           05  WS-HASHLINK-COUNT      PIC 9(09) VALUE ZERO.

      * Bounded in-memory tables for duplicate detection.  Fixed upper
      * bound; a ledger segment larger than WS-MAX-RECORDS would need a
      * sort-based reconciliation program instead of this table-driven
      * approach.
       01  WS-MAX-RECORDS             PIC 9(06) VALUE 050000.
       01  WS-SEEN-TABLE.
           05  WS-SEEN-ENTRY OCCURS 50000 TIMES
                   INDEXED BY WS-SEEN-IDX.
               10  WS-SEEN-SEQUENCE   PIC 9(18).
               10  WS-SEEN-RECORD-ID  PIC X(64).
       01  WS-SEEN-COUNT               PIC 9(06) VALUE ZERO.
       01  WS-SCAN-IDX                 PIC 9(06) VALUE ZERO.
       01  WS-DUP-SEQ-FOUND-SW         PIC X(01) VALUE 'N'.
           88  WS-DUP-SEQ-FOUND        VALUE 'Y'.
       01  WS-DUP-ID-FOUND-SW          PIC X(01) VALUE 'N'.
           88  WS-DUP-ID-FOUND         VALUE 'Y'.

       01  WS-CHECK-PARMS.
           05  WS-CHECK-NAME-PARM     PIC X(12) VALUE SPACES.
           05  WS-CHECK-STATUS-PARM   PIC X(04) VALUE SPACES.

       01  WS-REPORT-DETAIL.
           05  RD-SEQUENCE             PIC 9(18).
           05  FILLER                  PIC X(02) VALUE SPACES.
           05  RD-RECORD-ID            PIC X(64).
           05  FILLER                  PIC X(02) VALUE SPACES.
           05  RD-CHECK-NAME           PIC X(12).
           05  FILLER                  PIC X(02) VALUE SPACES.
           05  RD-STATUS               PIC X(04).

       PROCEDURE DIVISION.
       0000-MAIN.
           PERFORM 1000-INITIALIZE
           PERFORM 2000-RECONCILE-UNTIL-EOF
               UNTIL WS-END-OF-LEDGER
           PERFORM 7000-WRITE-SUMMARY
           PERFORM 9000-TERMINATE
           STOP RUN.

       1000-INITIALIZE.
           OPEN INPUT WORM-LEDGER-FILE
           IF NOT WS-LEDGER-OK
               DISPLAY "RECONCILE-LEDGER: cannot open ledger, status "
                   WS-LEDGER-STATUS
               STOP RUN
           END-IF

           OPEN OUTPUT REPORT-FILE
           IF NOT WS-REPORT-OK
               DISPLAY "RECONCILE-LEDGER: cannot open report, status "
                   WS-REPORT-STATUS
               CLOSE WORM-LEDGER-FILE
               STOP RUN
           END-IF

           MOVE "WORM LEDGER RECONCILIATION REPORT" TO REPORT-LINE
           WRITE REPORT-LINE
           MOVE SPACES TO REPORT-LINE
           WRITE REPORT-LINE

           PERFORM 8100-READ-NEXT-LEDGER-RECORD.

       2000-RECONCILE-UNTIL-EOF.
           ADD 1 TO WS-RECORD-COUNT
           COMPUTE WS-EXPECTED-SEQUENCE = WS-PRIOR-SEQUENCE + 1

      * Check 1: sequence gap (skipped for the very first record, which
      * establishes the baseline; a chain must still start at sequence
      * 1 to pass overall - enforced via the gap check itself since
      * WS-PRIOR-SEQUENCE starts at zero, making expected-sequence 1).
           IF WR-SEQUENCE OF LEDGER-RECORD-LINE
                   NOT = WS-EXPECTED-SEQUENCE
               ADD 1 TO WS-SEQGAP-COUNT
               ADD 1 TO WS-ANOMALY-COUNT
               SET WS-OVERALL-FAIL TO TRUE
               MOVE "SEQ-GAP     " TO WS-CHECK-NAME-PARM
               MOVE "FAIL" TO WS-CHECK-STATUS-PARM
               PERFORM 6100-REPORT-DETAIL-LINE
           END-IF

      * Check 4: prev-hash chain link (skipped for the very first
      * record, which has no predecessor on file).
           IF NOT WS-IS-FIRST-RECORD
               IF WR-PREV-HASH OF LEDGER-RECORD-LINE
                       NOT = WS-PRIOR-RECORD-HASH
                   ADD 1 TO WS-HASHLINK-COUNT
                   ADD 1 TO WS-ANOMALY-COUNT
                   SET WS-OVERALL-FAIL TO TRUE
                   MOVE "HASH-LINK   " TO WS-CHECK-NAME-PARM
                   MOVE "FAIL" TO WS-CHECK-STATUS-PARM
                   PERFORM 6100-REPORT-DETAIL-LINE
               END-IF
           END-IF

      * Checks 2 and 3: duplicate sequence / duplicate record-id,
      * scanned against everything seen so far in the bounded table.
           PERFORM 5000-CHECK-DUPLICATES

           IF WS-SEEN-COUNT < WS-MAX-RECORDS
               ADD 1 TO WS-SEEN-COUNT
               SET WS-SEEN-IDX TO WS-SEEN-COUNT
               MOVE WR-SEQUENCE OF LEDGER-RECORD-LINE
                   TO WS-SEEN-SEQUENCE (WS-SEEN-IDX)
               MOVE WR-RECORD-ID OF LEDGER-RECORD-LINE
                   TO WS-SEEN-RECORD-ID (WS-SEEN-IDX)
           ELSE
               DISPLAY "RECONCILE-LEDGER: WS-MAX-RECORDS exceeded - "
                   "duplicate detection incomplete beyond this point"
               ADD 1 TO WS-ANOMALY-COUNT
               SET WS-OVERALL-FAIL TO TRUE
           END-IF

           MOVE WR-SEQUENCE OF LEDGER-RECORD-LINE TO WS-PRIOR-SEQUENCE
           MOVE WR-RECORD-HASH OF LEDGER-RECORD-LINE
               TO WS-PRIOR-RECORD-HASH
           MOVE 'N' TO WS-FIRST-RECORD-SW

           PERFORM 8100-READ-NEXT-LEDGER-RECORD.

       5000-CHECK-DUPLICATES.
           MOVE 'N' TO WS-DUP-SEQ-FOUND-SW
           MOVE 'N' TO WS-DUP-ID-FOUND-SW

           PERFORM VARYING WS-SCAN-IDX FROM 1 BY 1
                   UNTIL WS-SCAN-IDX > WS-SEEN-COUNT
               IF WS-SEEN-SEQUENCE (WS-SCAN-IDX)
                       = WR-SEQUENCE OF LEDGER-RECORD-LINE
                   SET WS-DUP-SEQ-FOUND TO TRUE
               END-IF
               IF WS-SEEN-RECORD-ID (WS-SCAN-IDX)
                       = WR-RECORD-ID OF LEDGER-RECORD-LINE
                   SET WS-DUP-ID-FOUND TO TRUE
               END-IF
           END-PERFORM

           IF WS-DUP-SEQ-FOUND
               ADD 1 TO WS-DUPSEQ-COUNT
               ADD 1 TO WS-ANOMALY-COUNT
               SET WS-OVERALL-FAIL TO TRUE
               MOVE "DUP-SEQ     " TO WS-CHECK-NAME-PARM
               MOVE "FAIL" TO WS-CHECK-STATUS-PARM
               PERFORM 6100-REPORT-DETAIL-LINE
           END-IF

           IF WS-DUP-ID-FOUND
               ADD 1 TO WS-DUPID-COUNT
               ADD 1 TO WS-ANOMALY-COUNT
               SET WS-OVERALL-FAIL TO TRUE
               MOVE "DUP-RECID   " TO WS-CHECK-NAME-PARM
               MOVE "FAIL" TO WS-CHECK-STATUS-PARM
               PERFORM 6100-REPORT-DETAIL-LINE
           END-IF.

       6100-REPORT-DETAIL-LINE.
           MOVE SPACES TO WS-REPORT-DETAIL
           MOVE WR-SEQUENCE OF LEDGER-RECORD-LINE TO RD-SEQUENCE
           MOVE WR-RECORD-ID OF LEDGER-RECORD-LINE TO RD-RECORD-ID
           MOVE WS-CHECK-NAME-PARM TO RD-CHECK-NAME
           MOVE WS-CHECK-STATUS-PARM TO RD-STATUS
      * (parm fields set by caller immediately before this PERFORM)
           MOVE WS-REPORT-DETAIL TO REPORT-LINE
           WRITE REPORT-LINE.

       8100-READ-NEXT-LEDGER-RECORD.
           READ WORM-LEDGER-FILE
               AT END
                   SET WS-END-OF-LEDGER TO TRUE
           END-READ.

       7000-WRITE-SUMMARY.
           MOVE SPACES TO REPORT-LINE
           WRITE REPORT-LINE
           MOVE "SUMMARY" TO REPORT-LINE
           WRITE REPORT-LINE

           STRING "Records examined : " WS-RECORD-COUNT
               DELIMITED BY SIZE INTO REPORT-LINE
           WRITE REPORT-LINE

           STRING "Sequence gaps    : " WS-SEQGAP-COUNT
               DELIMITED BY SIZE INTO REPORT-LINE
           WRITE REPORT-LINE

           STRING "Duplicate seq    : " WS-DUPSEQ-COUNT
               DELIMITED BY SIZE INTO REPORT-LINE
           WRITE REPORT-LINE

           STRING "Duplicate rec-id : " WS-DUPID-COUNT
               DELIMITED BY SIZE INTO REPORT-LINE
           WRITE REPORT-LINE

           STRING "Hash-link breaks : " WS-HASHLINK-COUNT
               DELIMITED BY SIZE INTO REPORT-LINE
           WRITE REPORT-LINE

           STRING "Total anomalies  : " WS-ANOMALY-COUNT
               DELIMITED BY SIZE INTO REPORT-LINE
           WRITE REPORT-LINE

           MOVE SPACES TO REPORT-LINE
           WRITE REPORT-LINE

           IF WS-OVERALL-PASS
               MOVE "OVERALL RESULT: PASS" TO REPORT-LINE
           ELSE
               MOVE "OVERALL RESULT: FAIL" TO REPORT-LINE
           END-IF
           WRITE REPORT-LINE

           DISPLAY "RECONCILE-LEDGER: examined=" WS-RECORD-COUNT
               " anomalies=" WS-ANOMALY-COUNT
               " result=" WS-OVERALL-RESULT-SW.

       9000-TERMINATE.
           CLOSE WORM-LEDGER-FILE
           CLOSE REPORT-FILE.
