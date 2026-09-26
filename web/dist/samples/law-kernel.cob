       IDENTIFICATION DIVISION.
       PROGRAM-ID. COBOL-LAW-KERNEL.

       ENVIRONMENT DIVISION.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  WS-CASE-ID                PIC X(32).
       01  WS-CLAIM-TYPE             PIC X(24).
       01  WS-LEDGER-CODE            PIC X(24).
       01  WS-NOTICE-CODE            PIC X(24).
       01  WS-ROUTE                  PIC X(32).

       PROCEDURE DIVISION.
       MAIN-LOGIC.
           DISPLAY "COBOL LAW KERNEL ONLINE".
           DISPLAY "PURPOSE: CLAIM LEDGERS, TRUST ACCOUNT STATEMENTS, NOTICE BATCHES".
           DISPLAY "INPUT CASE ID:".
           ACCEPT WS-CASE-ID.
           DISPLAY "INPUT CLAIM TYPE:".
           ACCEPT WS-CLAIM-TYPE.

           EVALUATE WS-CLAIM-TYPE
               WHEN "BENEFICIARY"
                   MOVE "TRUST-BENEFICIARY" TO WS-LEDGER-CODE
                   MOVE "NOTICE-REPRESENTATION" TO WS-NOTICE-CODE
                   MOVE "FIDUCIARY-REVIEW" TO WS-ROUTE
               WHEN "ACH"
                   MOVE "ACH-DISPUTE" TO WS-LEDGER-CODE
                   MOVE "NOTICE-REVOCATION" TO WS-NOTICE-CODE
                   MOVE "RDFI-ODFI-PATH" TO WS-ROUTE
               WHEN "FCRA"
                   MOVE "CREDIT-DISPUTE" TO WS-LEDGER-CODE
                   MOVE "NOTICE-METRO2" TO WS-NOTICE-CODE
                   MOVE "FURNISHER-CRA-PATH" TO WS-ROUTE
               WHEN OTHER
                   MOVE "GENERAL-LEGAL" TO WS-LEDGER-CODE
                   MOVE "NOTICE-GENERAL" TO WS-NOTICE-CODE
                   MOVE "COUNSEL-REVIEW" TO WS-ROUTE
           END-EVALUATE.

           DISPLAY "CASE-ID: " WS-CASE-ID
           DISPLAY "LEDGER:  " WS-LEDGER-CODE
           DISPLAY "NOTICE:  " WS-NOTICE-CODE
           DISPLAY "ROUTE:   " WS-ROUTE
           GOBACK.
