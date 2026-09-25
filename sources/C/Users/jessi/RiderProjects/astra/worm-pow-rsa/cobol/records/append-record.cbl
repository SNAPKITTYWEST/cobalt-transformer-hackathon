      ******************************************************************
      * APPEND-RECORD.CBL
      *
      * Appends a single pre-validated record to the WORM ledger file.
      * Enforces append-only semantics at the file-access level: the
      * ledger file is OPENed in EXTEND mode only.  This program never
      * OPENs the ledger I-O, and therefore cannot REWRITE or DELETE any
      * existing record.  This is the sole sanctioned write path into
      * the ledger for a single incoming record; batch ingestion
      * (see cobol/ingestion) drives this same append logic in bulk.
      *
      * No cryptographic verification is performed here.  The caller is
      * responsible for having validated payload-hash / pow-hash /
      * record-hash upstream (Ada/SPARK).  WR-WORM-APPEND-ONLY and
      * WR-WORM-IMMUTABLE are expected to already be set to 'Y' on any
      * record destined for this ledger; this program enforces that
      * expectation defensively before writing.
      ******************************************************************
       IDENTIFICATION DIVISION.
       PROGRAM-ID. APPEND-RECORD.
       AUTHOR. SOVEREIGN-ENGINE-V2.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. GENERIC.
       OBJECT-COMPUTER. GENERIC.

       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT INCOMING-RECORD-FILE ASSIGN TO "INCOMREC"
               ORGANIZATION IS LINE SEQUENTIAL
               FILE STATUS IS WS-INCOMING-STATUS.

           SELECT WORM-LEDGER-FILE ASSIGN TO "WORMLDGR"
               ORGANIZATION IS LINE SEQUENTIAL
               FILE STATUS IS WS-LEDGER-STATUS.

       DATA DIVISION.
       FILE SECTION.
       FD  INCOMING-RECORD-FILE
           RECORDING MODE IS F.
       01  INCOMING-RECORD-LINE.
           COPY WORM-RECORD.

       FD  WORM-LEDGER-FILE
           RECORDING MODE IS F.
       01  LEDGER-RECORD-LINE.
           COPY WORM-RECORD.

       WORKING-STORAGE SECTION.
       01  WS-INCOMING-STATUS         PIC X(02) VALUE SPACES.
           88  WS-INCOMING-OK         VALUE '00'.
           88  WS-INCOMING-EOF        VALUE '10'.

       01  WS-LEDGER-STATUS           PIC X(02) VALUE SPACES.
           88  WS-LEDGER-OK           VALUE '00'.

       01  WS-COUNTERS.
           05  WS-RECORDS-READ        PIC 9(09) VALUE ZERO.
           05  WS-RECORDS-APPENDED    PIC 9(09) VALUE ZERO.
           05  WS-RECORDS-SKIPPED     PIC 9(09) VALUE ZERO.

       01  WS-FLAGS.
           05  WS-EOF-SWITCH          PIC X(01) VALUE 'N'.
               88  WS-END-OF-FILE     VALUE 'Y'.
           05  WS-VALID-FOR-APPEND    PIC X(01) VALUE 'Y'.
               88  WS-RECORD-VALID    VALUE 'Y'.
               88  WS-RECORD-INVALID  VALUE 'N'.

       PROCEDURE DIVISION.
       0000-MAIN.
           PERFORM 1000-INITIALIZE
           PERFORM 2000-PROCESS-UNTIL-EOF
               UNTIL WS-END-OF-FILE
           PERFORM 9000-TERMINATE
           STOP RUN.

       1000-INITIALIZE.
           OPEN INPUT INCOMING-RECORD-FILE
           IF NOT WS-INCOMING-OK
               DISPLAY "APPEND-RECORD: cannot open incoming file, status "
                   WS-INCOMING-STATUS
               STOP RUN
           END-IF

           OPEN EXTEND WORM-LEDGER-FILE
           IF NOT WS-LEDGER-OK
               DISPLAY "APPEND-RECORD: cannot open ledger for EXTEND, "
                   "status " WS-LEDGER-STATUS
               CLOSE INCOMING-RECORD-FILE
               STOP RUN
           END-IF

           PERFORM 8000-READ-NEXT-INCOMING.

       2000-PROCESS-UNTIL-EOF.
           SET WR-RECORD-VALID TO TRUE

           IF WR-WORM-APPEND-ONLY OF INCOMING-RECORD-LINE NOT = 'Y'
               SET WS-RECORD-INVALID TO TRUE
           END-IF

           IF WR-WORM-IMMUTABLE OF INCOMING-RECORD-LINE NOT = 'Y'
               SET WS-RECORD-INVALID TO TRUE
           END-IF

           IF WS-RECORD-VALID
               MOVE INCOMING-RECORD-LINE TO LEDGER-RECORD-LINE
               WRITE LEDGER-RECORD-LINE
               IF WS-LEDGER-OK
                   ADD 1 TO WS-RECORDS-APPENDED
               ELSE
                   DISPLAY "APPEND-RECORD: WRITE failed, status "
                       WS-LEDGER-STATUS
                   ADD 1 TO WS-RECORDS-SKIPPED
               END-IF
           ELSE
               DISPLAY "APPEND-RECORD: rejected record-id "
                   WR-RECORD-ID OF INCOMING-RECORD-LINE
                   " - append-only/immutable flags not set"
               ADD 1 TO WS-RECORDS-SKIPPED
           END-IF

           PERFORM 8000-READ-NEXT-INCOMING.

       8000-READ-NEXT-INCOMING.
           READ INCOMING-RECORD-FILE
               AT END
                   SET WS-END-OF-FILE TO TRUE
               NOT AT END
                   ADD 1 TO WS-RECORDS-READ
           END-READ.

       9000-TERMINATE.
           CLOSE INCOMING-RECORD-FILE
           CLOSE WORM-LEDGER-FILE
           DISPLAY "APPEND-RECORD: read=" WS-RECORDS-READ
               " appended=" WS-RECORDS-APPENDED
               " skipped=" WS-RECORDS-SKIPPED.
