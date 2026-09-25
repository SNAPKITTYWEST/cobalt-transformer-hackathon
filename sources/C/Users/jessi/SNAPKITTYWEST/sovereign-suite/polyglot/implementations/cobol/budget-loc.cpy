       *> ═══════════════════════════════════════════════════════════
       *> SNAPKITTY OS - POLYGLOT BUDGET VARIANCE ENGINE
       *> COBOL Copybook: Budget vs Actual
       *> ═══════════════════════════════════════════════════════════

       01  BUDGET-RECORD.
           05 BUDGET-ID              PIC X(36).
           05 BUDGET-NAME            PIC X(80).
           05 BUDGET-ENTITY-ID       PIC X(36).
           05 BUDGET-FISCAL-YEAR     PIC 9(4).
           05 BUDGET-STATUS          PIC X(10).
              88 BUDGET-DRAFT        VALUE 'DRAFT'.
              88 BUDGET-APPROVED     VALUE 'APPROVED'.
              88 BUDGET-ACTIVE       VALUE 'ACTIVE'.
              88 BUDGET-CLOSED       VALUE 'CLOSED'.
           05 BUDGET-TOTAL-CENTS     PIC 9(15)V99.

       01  BUDGET-LINE-ITEM-RECORD.
           05 BLINE-ID               PIC X(36).
           05 BLINE-BUDGET-ID        PIC X(36).
           05 BLINE-ACCOUNT-ID       PIC X(36).
           05 BLINE-PERIOD           PIC X(7).
           05 BLINE-BUDGET-CENTS     PIC 9(15)V99.
           05 BLINE-NOTES            PIC X(200).

       01  BUDGET-VARIANCE-RECORD.
           05 BVAR-ID                PIC X(36).
           05 BVAR-BUDGET-ID         PIC X(36).
           05 BVAR-ACCOUNT-ID        PIC X(36).
           05 BVAR-ACCOUNT-CODE      PIC X(10).
           05 BVAR-ACCOUNT-NAME      PIC X(80).
           05 BVAR-PERIOD            PIC X(7).
           05 BVAR-BUDGET-CENTS      PIC 9(15)V99.
           05 BVAR-ACTUAL-CENTS      PIC 9(15)V99.
           05 BVAR-VARIANCE-CENTS    PIC S9(15)V99.
           05 BVAR-VARIANCE-PCT      PIC S9(3)V99.
           05 BVAR-IS-OVER-BUDGET    PIC X(1).
              88 OVER-BUDGET         VALUE 'Y'.
              88 NOT-OVER-BUDGET     VALUE 'N'.
           05 BVAR-FLAG              PIC X(8).
              88 FLAG-CRITICAL       VALUE 'CRITICAL'.
              88 FLAG-WARNING        VALUE 'WARNING'.
              88 FLAG-OK             VALUE 'OK'.

       01  VARIANCE-THRESHOLD-RECORD.
           05 VTHR-CRITICAL-PCT      PIC 9(3)V99.
           05 VTHR-WARNING-PCT       PIC 9(3)V99.
           05 VTHR-TOLERANCE-CENTS   PIC 9(15)V99.

       01  VARIANCE-REPORT-RECORD.
           05 VREP-BUDGET-ID         PIC X(36).
           05 VREP-PERIOD            PIC X(7).
           05 VREP-TOTAL-BUDGET      PIC 9(18)V99.
           05 VREP-TOTAL-ACTUAL      PIC 9(18)V99.
           05 VREP-TOTAL-VARIANCE    PIC S9(18)V99.
           05 VREP-OVER-COUNT        PIC 9(3).
           05 VREP-UNDER-COUNT       PIC 9(3).
           05 VREP-ON-TRACK-COUNT    PIC 9(3).
           05 VREP-CRITICAL-COUNT    PIC 9(3).

       01  LOC-AUDIT-RECORD.
           05 LOC-ID                 PIC X(36).
           05 LOC-ACCOUNT-NUMBER     PIC X(20).
           05 LOC-LENDER-NAME        PIC X(80).
           05 LOC-CREDIT-LIMIT       PIC 9(15)V99.
           05 LOC-AVAILABLE          PIC 9(15)V99.
           05 LOC-USED               PIC 9(15)V99.
           05 LOC-INTEREST-RATE-BPS  PIC 9(4).
           05 LOC-RATE-TYPE          PIC X(10).
              88 RATE-FIXED          VALUE 'FIXED'.
              88 RATE-VARIABLE       VALUE 'VARIABLE'.
              88 RATE-PRIME-PLUS     VALUE 'PRIME_PLUS'.
           05 LOC-MATURITY-DATE      PIC X(10).
           05 LOC-STATUS             PIC X(10).
              88 LOC-ACTIVE          VALUE 'ACTIVE'.
              88 LOC-FROZEN          VALUE 'FROZEN'.
              88 LOC-DEFAULT         VALUE 'DEFAULT'.
              88 LOC-PAID-OFF        VALUE 'PAID_OFF'.
              88 LOC-EXPIRED         VALUE 'EXPIRED'.

       01  COVENANT-CHECK-RECORD.
           05 COV-TYPE               PIC X(20).
           05 COV-THRESHOLD          PIC 9(3)V99.
           05 COV-ACTUAL             PIC 9(3)V99.
           05 COV-STATUS             PIC X(8).
              88 COV-PASS            VALUE 'PASS'.
              88 COV-FAIL            VALUE 'FAIL'.
              88 COV-WAIVED          VALUE 'WAIVED'.
           05 COV-TREND              PIC X(12).
              88 TREND-IMPROVING     VALUE 'IMPROVING'.
              88 TREND-STABLE        VALUE 'STABLE'.
              88 TREND-DECLINING     VALUE 'DECLINING'.
