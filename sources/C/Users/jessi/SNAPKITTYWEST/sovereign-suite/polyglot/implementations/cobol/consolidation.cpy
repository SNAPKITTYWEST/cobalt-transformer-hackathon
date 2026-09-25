       *> ═══════════════════════════════════════════════════════════
       *> SNAPKITTY OS - POLYGLOT CONSOLIDATION ENGINE
       *> COBOL Copybook: Multi-Entity Consolidation
       *> ═══════════════════════════════════════════════════════════

       01  ENTITY-RECORD.
           05 ENTITY-ID              PIC X(36).
           05 ENTITY-CODE            PIC X(10).
           05 ENTITY-NAME            PIC X(80).
           05 ENTITY-TYPE            PIC X(12).
              88 ENTITY-NONPROFIT    VALUE 'NONPROFIT'.
              88 ENTITY-BCORP        VALUE 'BCORP'.
              88 ENTITY-TRUST        VALUE 'TRUST'.
              05 ENTITY-PARENT-ID    PIC X(36).
           05 ENTITY-CURRENCY        PIC X(3).
           05 ENTITY-STATUS          PIC X(8).
              88 ENTITY-ACTIVE       VALUE 'ACTIVE'.
              88 ENTITY-INACTIVE     VALUE 'INACTIVE'.

       01  IC-TRANSACTION-RECORD.
           05 IC-TXN-ID              PIC X(36).
           05 IC-TXN-TYPE            PIC X(16).
              88 IC-LOAN             VALUE 'LOAN'.
              88 IC-ALLOCATION       VALUE 'ALLOCATION'.
              88 IC-SERVICE-FEE      VALUE 'SERVICE_FEE'.
              88 IC-DIVIDEND         VALUE 'DIVIDEND'.
              88 IC-TRANSFER         VALUE 'TRANSFER'.
           05 IC-FROM-ENTITY         PIC X(36).
           05 IC-TO-ENTITY           PIC X(36).
           05 IC-AMOUNT              PIC 9(15)V99.
           05 IC-DESCRIPTION         PIC X(200).
           05 IC-REFERENCE           PIC X(50).
           05 IC-STATUS              PIC X(12).
              88 IC-PENDING          VALUE 'PENDING'.
              88 IC-APPROVED         VALUE 'APPROVED'.
              88 IC-ELIMINATED       VALUE 'ELIMINATED'.
              88 IC-VOID             VALUE 'VOID'.

       01  CONSOLIDATION-RULE-RECORD.
           05 RULE-ID                PIC X(36).
           05 RULE-TYPE              PIC X(20).
              88 RULE-ELIMINATION    VALUE 'ELIMINATION'.
              88 RULE-INTERCOMPANY   VALUE 'INTERCOMPANY'.
              88 RULE-CURRENCY-TRANS VALUE 'CURRENCY_TRANSLATION'.
              88 RULE-RECLASS        VALUE 'RECLASSIFICATION'.
           05 RULE-NAME              PIC X(80).
           05 RULE-FROM-ACCT         PIC X(10).
           05 RULE-TO-ACCT           PIC X(10).
           05 RULE-PERCENTAGE        PIC 9(3)V99.

       01  CONSOLIDATION-RESULT-RECORD.
           05 CONSOL-PERIOD          PIC X(7).
           05 CONSOL-ENTITIES-COUNT  PIC 9(3).
           05 CONSOL-ELIM-COUNT      PIC 9(3).
           05 CONSOL-TOTAL-BALANCE   PIC 9(18)V99.
           05 CONSOL-STATUS          PIC X(12).
              88 CONSOL-DRAFT        VALUE 'DRAFT'.
              88 CONSOL-COMPLETED    VALUE 'COMPLETED'.
              88 CONSOL-FAILED       VALUE 'FAILED'.

       01  CURRENCY-TRANSLATION-RECORD.
           05 CURR-FROM              PIC X(3).
           05 CURR-TO                PIC X(3).
           05 CURR-RATE              PIC 9(3)V9(8).
           05 CURR-EFFECTIVE-DATE    PIC X(10).
           05 CURR-SOURCE            PIC X(20).
