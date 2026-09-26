      *> VAULT TREASURY ENGINE — GnuCOBOL free format
      *> cobc -x -free -o vault-treasury vault_treasury.cbl
      *> VAULT axiom: capital never moves without a seal.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. VAULT-TREASURY.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-QUERY             PIC X(200) VALUE SPACES.
       01 WS-STATUS            PIC X(10)  VALUE 'SEALED'.
       01 WS-RESERVE-CHECK     PIC X(4)   VALUE 'PASS'.
       01 WS-APPROVAL          PIC X(3)   VALUE 'YES'.
       01 WS-VETO              PIC X(3)   VALUE 'YES'.
       01 WS-RESERVE           PIC 9(3)   VALUE 75.
       01 WS-THRESHOLD         PIC 9(3)   VALUE 20.
       01 WS-DECISION          PIC X(200) VALUE SPACES.
       01 WS-Q-LOWER           PIC X(200) VALUE SPACES.

       PROCEDURE DIVISION.
       MAIN-PARA.
           ACCEPT WS-QUERY FROM STDIN

           MOVE FUNCTION LOWER-CASE(WS-QUERY) TO WS-Q-LOWER

           PERFORM CHECK-RESERVE
           PERFORM DETECT-INTENT
           PERFORM BUILD-DECISION
           PERFORM EMIT-OUTPUT

           STOP RUN.

       CHECK-RESERVE.
           IF WS-RESERVE <= WS-THRESHOLD
               MOVE 'FAIL'   TO WS-RESERVE-CHECK
               MOVE 'FROZEN' TO WS-STATUS
           END-IF.

       DETECT-INTENT.
           IF WS-Q-LOWER(1:6) = 'freeze'
               MOVE 'FROZEN' TO WS-STATUS
           END-IF
           IF WS-Q-LOWER(1:7) = 'approve'
               IF WS-RESERVE-CHECK = 'PASS'
                   MOVE 'SEALED' TO WS-STATUS
               END-IF
           END-IF
           IF WS-Q-LOWER(1:7) = 'release'
               IF WS-RESERVE-CHECK = 'FAIL'
                   MOVE 'FROZEN' TO WS-STATUS
               END-IF
           END-IF.

       BUILD-DECISION.
           EVALUATE WS-STATUS
               WHEN 'SEALED'
                   MOVE 'Reserve threshold met. Capital movement authorized. Dual signature pending.'
                       TO WS-DECISION
               WHEN 'FROZEN'
                   MOVE 'Reserve below threshold. Capital frozen. VAULT veto active.'
                       TO WS-DECISION
               WHEN OTHER
                   MOVE 'Treasury online. Reserves nominal. VAULT seal operational.'
                       TO WS-DECISION
           END-EVALUATE.

       EMIT-OUTPUT.
           DISPLAY 'agent=vault'
           DISPLAY 'status=' FUNCTION TRIM(WS-STATUS)
           DISPLAY 'reserve_check=' FUNCTION TRIM(WS-RESERVE-CHECK)
           DISPLAY 'approval_required=' FUNCTION TRIM(WS-APPROVAL)
           DISPLAY 'veto_power=' FUNCTION TRIM(WS-VETO)
           DISPLAY 'decision=' FUNCTION TRIM(WS-DECISION)
           DISPLAY 'certified=true'
           DISPLAY 'engine=cobol-vault-treasury'.
