      ******************************************************************
      * WORM-RECORD.CPY
      * Shared copybook for the WORM (write-once-read-many) cryptographic
      * continuity ledger.  Defines the fixed-format record layout used
      * by all COBOL programs that ingest, append, archive, or reconcile
      * ledger records.
      *
      * IMPORTANT:
      *   - All cryptographic validation (payload hashing, PoW hash
      *     verification, RSA/signature checks) happens UPSTREAM in
      *     Ada/SPARK.  COBOL never recomputes hashes; it only performs
      *     string-equality comparisons and sequence/structural checks.
      *   - This layout assumes fixed-length, line-sequential records.
      *   - "Hex" fields are stored as 64-character alphanumeric strings
      *     (PIC X), not as packed/binary hash values.
      ******************************************************************

       01  WORM-LEDGER-RECORD.
           05  WR-SCHEMA-ID            PIC X(08).
           05  WR-SEQUENCE             PIC 9(18).
           05  WR-RECORD-ID            PIC X(64).
           05  WR-TIMESTAMP            PIC X(26).
           05  WR-EVENT-TYPE           PIC X(20).
           05  WR-PAYLOAD              PIC X(2000).
           05  WR-PAYLOAD-HASH         PIC X(64).
           05  WR-PREV-HASH            PIC X(64).
           05  WR-POW-ALGORITHM        PIC X(16).
           05  WR-POW-NONCE            PIC 9(18).
           05  WR-POW-DIFFICULTY       PIC 9(09).
           05  WR-POW-HASH             PIC X(64).
           05  WR-RECORD-HASH          PIC X(64).
           05  WR-WORM-MODE            PIC X(12).
           05  WR-WORM-APPEND-ONLY     PIC X(01).
               88  WR-APPEND-ONLY-YES  VALUE 'Y'.
               88  WR-APPEND-ONLY-NO   VALUE 'N'.
           05  WR-WORM-IMMUTABLE       PIC X(01).
               88  WR-IMMUTABLE-YES    VALUE 'Y'.
               88  WR-IMMUTABLE-NO     VALUE 'N'.
           05  FILLER                  PIC X(10).

      ******************************************************************
      * End of WORM-LEDGER-RECORD definition.
      * Total record length below is illustrative; recompute if fields
      * change.  (8+18+64+26+20+2000+64+64+16+18+9+64+64+12+1+1+10)
      ******************************************************************
