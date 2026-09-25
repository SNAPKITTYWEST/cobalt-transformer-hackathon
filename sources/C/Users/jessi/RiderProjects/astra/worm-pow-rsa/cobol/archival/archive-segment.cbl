      ******************************************************************
      * ARCHIVE-SEGMENT.CBL
      *
      * Archives a completed WORM ledger segment by copying it, in
      * order, to an archival output file prefixed with a manifest
      * header record.  The manifest carries: chain id, record count,
      * first/last sequence number, and first/last record-hash for the
      * segment being archived.
      *
      * The archival file is written once, sequentially, via OPEN
      * OUTPUT.  After this program completes, the archival file is
      * considered read-only; no program in this suite ever re-opens an
      * archival file for write/extend.  The source ledger file is only
      * ever read here (OPEN INPUT) - archival never rewrites the
      * ledger.
      *
      * No cryptographic recomputation is performed; hashes are copied
      * verbatim from the ledger records as produced upstream.
      ******************************************************************
       IDENTIFICATION DIVISION.
       PROGRAM-ID. ARCHIVE-SEGMENT.
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

           SELECT ARCHIVE-FILE ASSIGN TO "ARCHIVE"
               ORGANIZATION IS LINE SEQUENTIAL
               FILE STATUS IS WS-ARCHIVE-STATUS.

       DATA DIVISION.
       FILE SECTION.
       FD  WORM-LEDGER-FILE
           RECORDING MODE IS F.
       01  LEDGER-RECORD-LINE.
           COPY WORM-RECORD.

       FD  ARCHIVE-FILE
           RECORDING MODE IS F.
       01  ARCHIVE-RECORD-LINE         PIC X(2500).

       WORKING-STORAGE SECTION.
       01  WS-LEDGER-STATUS           PIC X(02) VALUE SPACES.
           88  WS-LEDGER-OK           VALUE '00'.
           88  WS-LEDGER-EOF          VALUE '10'.

       01  WS-ARCHIVE-STATUS          PIC X(02) VALUE SPACES.
           88  WS-ARCHIVE-OK          VALUE '00'.

       01  WS-SWITCHES.
           05  WS-EOF-SW              PIC X(01) VALUE 'N'.
               88  WS-END-OF-LEDGER   VALUE 'Y'.
           05  WS-FIRST-RECORD-SW     PIC X(01) VALUE 'Y'.
               88  WS-IS-FIRST-RECORD VALUE 'Y'.

       01  WS-MANIFEST.
           05  WS-CHAIN-ID            PIC X(20) VALUE SPACES.
           05  WS-RECORD-COUNT        PIC 9(09) VALUE ZERO.
           05  WS-FIRST-SEQUENCE      PIC 9(18) VALUE ZERO.
           05  WS-LAST-SEQUENCE       PIC 9(18) VALUE ZERO.
           05  WS-FIRST-HASH          PIC X(64) VALUE SPACES.
           05  WS-LAST-HASH           PIC X(64) VALUE SPACES.

      * Manifest header layout as written to the archive file. The
      * archive file therefore contains one manifest header record
      * followed by every archived ledger record, in original order.
       01  WS-MANIFEST-HEADER-LINE.
           05  MH-TAG                 PIC X(10) VALUE "MANIFEST01".
           05  MH-CHAIN-ID            PIC X(20).
           05  MH-RECORD-COUNT        PIC 9(09).
           05  MH-FIRST-SEQUENCE      PIC 9(18).
           05  MH-LAST-SEQUENCE       PIC 9(18).
           05  MH-FIRST-HASH          PIC X(64).
           05  MH-LAST-HASH           PIC X(64).
           05  MH-FILLER              PIC X(2295) VALUE SPACES.

       01  WS-CHAIN-ID-PARM           PIC X(20) VALUE "DEFAULT-CHAIN".

       PROCEDURE DIVISION.
      * The manifest header must be the first line of the archive file,
      * but its record-count/first/last fields cannot be known until
      * the whole ledger has been read.  Under LINE SEQUENTIAL there is
      * no REWRITE of an already-written line, so this program makes
      * two passes over the (read-only) ledger file: pass 1 computes
      * the manifest values, pass 2 writes the manifest header followed
      * by every ledger record, all within a single OPEN OUTPUT on the
      * archive file (archive file is opened for output exactly once;
      * it is never re-opened for write afterward).
       0000-MAIN.
           MOVE WS-CHAIN-ID-PARM TO WS-CHAIN-ID
           PERFORM 1000-PASS-ONE-COMPUTE-MANIFEST
           PERFORM 2000-PASS-TWO-WRITE-ARCHIVE
           PERFORM 9000-TERMINATE
           STOP RUN.

       1000-PASS-ONE-COMPUTE-MANIFEST.
           OPEN INPUT WORM-LEDGER-FILE
           IF NOT WS-LEDGER-OK
               DISPLAY "ARCHIVE-SEGMENT: cannot open ledger, status "
                   WS-LEDGER-STATUS
               STOP RUN
           END-IF

           PERFORM UNTIL WS-END-OF-LEDGER
               READ WORM-LEDGER-FILE
                   AT END
                       SET WS-END-OF-LEDGER TO TRUE
                   NOT AT END
                       ADD 1 TO WS-RECORD-COUNT
                       IF WS-IS-FIRST-RECORD
                           MOVE WR-SEQUENCE OF LEDGER-RECORD-LINE
                               TO WS-FIRST-SEQUENCE
                           MOVE WR-RECORD-HASH OF LEDGER-RECORD-LINE
                               TO WS-FIRST-HASH
                           MOVE 'N' TO WS-FIRST-RECORD-SW
                       END-IF
                       MOVE WR-SEQUENCE OF LEDGER-RECORD-LINE
                           TO WS-LAST-SEQUENCE
                       MOVE WR-RECORD-HASH OF LEDGER-RECORD-LINE
                           TO WS-LAST-HASH
               END-READ
           END-PERFORM
           CLOSE WORM-LEDGER-FILE
           MOVE 'N' TO WS-EOF-SW.

       2000-PASS-TWO-WRITE-ARCHIVE.
           OPEN INPUT WORM-LEDGER-FILE
           IF NOT WS-LEDGER-OK
               DISPLAY "ARCHIVE-SEGMENT: cannot reopen ledger, status "
                   WS-LEDGER-STATUS
               STOP RUN
           END-IF

           OPEN OUTPUT ARCHIVE-FILE
           IF NOT WS-ARCHIVE-OK
               DISPLAY "ARCHIVE-SEGMENT: cannot open archive, status "
                   WS-ARCHIVE-STATUS
               CLOSE WORM-LEDGER-FILE
               STOP RUN
           END-IF

           PERFORM 3000-WRITE-MANIFEST-HEADER

           PERFORM 8100-READ-NEXT-LEDGER-RECORD
           PERFORM 2100-COPY-ONE-RECORD
               UNTIL WS-END-OF-LEDGER.

       2100-COPY-ONE-RECORD.
           MOVE SPACES TO ARCHIVE-RECORD-LINE
           MOVE LEDGER-RECORD-LINE TO ARCHIVE-RECORD-LINE
           WRITE ARCHIVE-RECORD-LINE
           IF NOT WS-ARCHIVE-OK
               DISPLAY "ARCHIVE-SEGMENT: archive WRITE failed, status "
                   WS-ARCHIVE-STATUS
           END-IF
           PERFORM 8100-READ-NEXT-LEDGER-RECORD.

       3000-WRITE-MANIFEST-HEADER.
           MOVE WS-CHAIN-ID       TO MH-CHAIN-ID
           MOVE WS-RECORD-COUNT   TO MH-RECORD-COUNT
           MOVE WS-FIRST-SEQUENCE TO MH-FIRST-SEQUENCE
           MOVE WS-LAST-SEQUENCE  TO MH-LAST-SEQUENCE
           MOVE WS-FIRST-HASH     TO MH-FIRST-HASH
           MOVE WS-LAST-HASH      TO MH-LAST-HASH
           WRITE ARCHIVE-RECORD-LINE FROM WS-MANIFEST-HEADER-LINE.

       8100-READ-NEXT-LEDGER-RECORD.
           READ WORM-LEDGER-FILE
               AT END
                   SET WS-END-OF-LEDGER TO TRUE
           END-READ.

       9000-TERMINATE.
           CLOSE WORM-LEDGER-FILE
           CLOSE ARCHIVE-FILE
           DISPLAY "ARCHIVE-SEGMENT: chain-id=" WS-CHAIN-ID
               " records=" WS-RECORD-COUNT
               " first-seq=" WS-FIRST-SEQUENCE
               " last-seq=" WS-LAST-SEQUENCE.
