       IDENTIFICATION DIVISION.
       PROGRAM-ID. REQPARSE.
       AUTHOR. COBOL-NUGGET-COMPILER.
       DATE-WRITTEN. 2026-09-25.
       REMARKS. Composed HTTP/1.1 request-line parser (nugget N07).
      * METHOD SP TARGET SP VERSION  — no body, no headers.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. GNUCOBOL.
       OBJECT-COMPUTER. GNUCOBOL.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  WS-INPUT-BUFFER             PIC X(512) VALUE SPACES.
       01  WS-BUFFER-LEN               PIC 9(4)   VALUE 0.
       01  WS-STATUS                   PIC X(12)  VALUE "INIT".
          88  ST-OK                    VALUE "APPROVED".
          88  ST-BAD                   VALUE "DECLINED".
          88  ST-INIT                  VALUE "INIT".

       01  WS-METHOD                   PIC X(16)  VALUE SPACES.
       01  WS-TARGET                   PIC X(256) VALUE SPACES.
       01  WS-VERSION                  PIC X(16)  VALUE SPACES.

       01  WS-I                        PIC 9(4)   VALUE 0.
       01  WS-START                    PIC 9(4)   VALUE 0.
       01  WS-SP-COUNT                 PIC 9(2)   VALUE 0.
       01  WS-CHAR                     PIC X      VALUE SPACE.
       01  WS-OK-FLAG                  PIC X      VALUE "Y".
          88  FLAG-OK                  VALUE "Y".
          88  FLAG-BAD                 VALUE "N".

       PROCEDURE DIVISION.
       MAIN.
           PERFORM INITIALIZE-SYSTEM
           PERFORM ACCEPT-INPUT
           PERFORM VALIDATE-BUFFER
           IF FLAG-OK
              PERFORM PARSE-REQUEST-LINE
           END-IF
           PERFORM VERIFY-RESULT
           PERFORM TERMINATE-SYSTEM
           STOP RUN.

       INITIALIZE-SYSTEM.
           MOVE SPACES TO WS-INPUT-BUFFER
           MOVE ZERO   TO WS-BUFFER-LEN
           MOVE "INIT" TO WS-STATUS
           MOVE "Y"    TO WS-OK-FLAG
           MOVE SPACES TO WS-METHOD WS-TARGET WS-VERSION
           MOVE ZERO   TO WS-I WS-START WS-SP-COUNT.

       ACCEPT-INPUT.
           ACCEPT WS-INPUT-BUFFER
           INSPECT WS-INPUT-BUFFER TALLYING WS-BUFFER-LEN
               FOR CHARACTERS BEFORE INITIAL X"00".
           IF WS-BUFFER-LEN = 0
              INSPECT WS-INPUT-BUFFER TALLYING WS-BUFFER-LEN
                  FOR CHARACTERS BEFORE INITIAL SPACE
           END-IF
           PERFORM VARYING WS-I FROM 512 BY -1 UNTIL WS-I < 1
              IF WS-INPUT-BUFFER(WS-I:1) NOT = SPACE
                 MOVE WS-I TO WS-BUFFER-LEN
                 MOVE 1 TO WS-I
              END-IF
           END-PERFORM.

       VALIDATE-BUFFER.
      * Nugget N01
           IF WS-BUFFER-LEN = 0
              MOVE "N" TO WS-OK-FLAG
              MOVE "DECLINED" TO WS-STATUS
           END-IF
           IF WS-BUFFER-LEN > 512
              MOVE "N" TO WS-OK-FLAG
              MOVE "DECLINED" TO WS-STATUS
           END-IF.

       PARSE-REQUEST-LINE.
           PERFORM FIND-TOKENS
           IF FLAG-OK
              PERFORM VALIDATE-METHOD
              PERFORM VALIDATE-TARGET
              PERFORM VALIDATE-VERSION
           END-IF
           IF FLAG-OK
              MOVE "APPROVED" TO WS-STATUS
           ELSE
              MOVE "DECLINED" TO WS-STATUS
           END-IF.

       FIND-TOKENS.
      * N02 + N03 + N04 + N05
           MOVE 1 TO WS-START
           MOVE ZERO TO WS-SP-COUNT
           PERFORM VARYING WS-I FROM 1 BY 1
              UNTIL WS-I > WS-BUFFER-LEN
              MOVE WS-INPUT-BUFFER(WS-I:1) TO WS-CHAR
              IF WS-CHAR = SPACE
                 ADD 1 TO WS-SP-COUNT
                 EVALUATE WS-SP-COUNT
                    WHEN 1
                       COMPUTE WS-BUFFER-LEN = WS-BUFFER-LEN
                       MOVE WS-INPUT-BUFFER(WS-START:WS-I - WS-START)
                          TO WS-METHOD
                       COMPUTE WS-START = WS-I + 1
                    WHEN 2
                       MOVE WS-INPUT-BUFFER(WS-START:WS-I - WS-START)
                          TO WS-TARGET
                       COMPUTE WS-START = WS-I + 1
                    WHEN OTHER
                       MOVE "N" TO WS-OK-FLAG
                 END-EVALUATE
              END-IF
           END-PERFORM
           IF WS-SP-COUNT < 2
              MOVE "N" TO WS-OK-FLAG
           ELSE
              MOVE WS-INPUT-BUFFER(WS-START:) TO WS-VERSION
           END-IF.

       VALIDATE-METHOD.
      * part of N03
           IF WS-METHOD = SPACES
              MOVE "N" TO WS-OK-FLAG
           END-IF
           IF WS-METHOD NOT = "GET"
              AND WS-METHOD NOT = "POST"
              AND WS-METHOD NOT = "HEAD"
              AND WS-METHOD NOT = "PUT"
              AND WS-METHOD NOT = "DELETE"
              MOVE "N" TO WS-OK-FLAG
           END-IF.

       VALIDATE-TARGET.
      * part of N04
           IF WS-TARGET = SPACES
              MOVE "N" TO WS-OK-FLAG
           END-IF
           IF WS-TARGET(1:1) NOT = "/"
              MOVE "N" TO WS-OK-FLAG
           END-IF.

       VALIDATE-VERSION.
      * N06
           IF WS-VERSION NOT = "HTTP/1.0"
              AND WS-VERSION NOT = "HTTP/1.1"
              MOVE "N" TO WS-OK-FLAG
           END-IF.

       VERIFY-RESULT.
           DISPLAY "STATUS=" WS-STATUS
           DISPLAY "METHOD=" WS-METHOD
           DISPLAY "TARGET=" WS-TARGET
           DISPLAY "VERSION=" WS-VERSION.

       TERMINATE-SYSTEM.
           CONTINUE.
