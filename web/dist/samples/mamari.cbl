       IDENTIFICATION DIVISION.
       PROGRAM-ID. MAMARI-TABLET-DECODER.
       AUTHOR. AHMAD-ALI-PARR.
      *================================================================
      * THE MAMARI TABLET — COBOL LUNAR CALENDAR PROCESSOR
      * Easter Island. ~800 CE. 30 confirmed lunar glyphs.
      * COBOL processes structured records. The Mamari Tablet IS
      * a structured record: 30 rows, each a lunar phase entry.
      * The ancient scribe and the COBOL programmer solved the same
      * problem: how do you process a fixed-length sequential record
      * and branch on threshold conditions?
      *
      * THE OISC THESIS: SUBLEQ over a lunar counter.
      * A = current phase (0-29)
      * B = threshold (15=full moon, 29=dark moon)
      * C = adjacent glyph (fish/bird = what to do at threshold)
      *================================================================

       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT MAMARI-FILE ASSIGN TO "MAMARI.DAT"
               ORGANIZATION IS LINE SEQUENTIAL.

       DATA DIVISION.
       FILE SECTION.
       FD MAMARI-FILE.
       01 GLYPH-RECORD.
           05 GLYPH-POSITION   PIC 9(2).
           05 GLYPH-ID         PIC X(6).
           05 LUNAR-PHASE      PIC X(12).
           05 ADJACENT-GLYPH   PIC X(6).
           05 CONFIDENCE       PIC 9V99.

       WORKING-STORAGE SECTION.
       01 WS-EOF               PIC X(1) VALUE 'N'.
       01 WS-MOON-COUNT        PIC 9(2) VALUE 0.
       01 WS-THRESHOLD-A       PIC 9(2) VALUE 15.
       01 WS-THRESHOLD-B       PIC 9(2) VALUE 29.
       01 WS-CURRENT-POSITION  PIC 9(2) VALUE 0.

      * THE BRANCH INSTRUCTION — what fires at threshold
       01 WS-C-OPERAND         PIC X(20).

      * OISC PRIMITIVE: SUBLEQ(A, B, C)
      * If A >= B: subtract B from A, jump to C
      * Same operation the ancient scribe encoded in every moon glyph
       01 WS-SUBLEQ-A          PIC 9(2).
       01 WS-SUBLEQ-B          PIC 9(2).
       01 WS-SUBLEQ-RESULT     PIC 9(2).

       PROCEDURE DIVISION.
       000-MAIN.
           OPEN INPUT MAMARI-FILE
           PERFORM 100-READ-GLYPH UNTIL WS-EOF = 'Y'
           DISPLAY "LUNAR CYCLE COMPLETE: " WS-MOON-COUNT " PHASES"
           DISPLAY "FULL MOON AT POSITION: " WS-THRESHOLD-A
           DISPLAY "DARK MOON AT POSITION: " WS-THRESHOLD-B
           DISPLAY "THE BRANCH FIRES: " WS-C-OPERAND
           CLOSE MAMARI-FILE
           STOP RUN.

       100-READ-GLYPH.
           READ MAMARI-FILE
               AT END MOVE 'Y' TO WS-EOF
               NOT AT END PERFORM 200-PROCESS-GLYPH
           END-READ.

       200-PROCESS-GLYPH.
           ADD 1 TO WS-MOON-COUNT
           MOVE GLYPH-POSITION TO WS-CURRENT-POSITION

      *    OISC: test if we have reached threshold
           MOVE WS-CURRENT-POSITION TO WS-SUBLEQ-A

           IF GLYPH-POSITION = WS-THRESHOLD-A
               DISPLAY "FULL MOON — PHASE TRANSITION"
               DISPLAY "ADJACENT (C OPERAND): " ADJACENT-GLYPH
               MOVE ADJACENT-GLYPH TO WS-C-OPERAND
               PERFORM 300-BRANCH-INSTRUCTION
           END-IF

           IF GLYPH-POSITION = WS-THRESHOLD-B
               DISPLAY "DARK MOON — PHASE TRANSITION"
               DISPLAY "ADJACENT (C OPERAND): " ADJACENT-GLYPH
               MOVE ADJACENT-GLYPH TO WS-C-OPERAND
               PERFORM 300-BRANCH-INSTRUCTION
           END-IF.

       300-BRANCH-INSTRUCTION.
      *================================================================
      * THE C OPERAND FIRES.
      * This is the fish or bird glyph — the ritual instruction.
      * Count the cycle. Know where you are.
      * When you reach the threshold — the adjacent glyph tells you what to do.
      *
      * Same logic as:
      *   - The Enochian 49th Call (C fires after 48 A-B pairs)
      *   - Proto-Elamite accounting (C = distribute when inventory = minimum)
      *   - Every ancient script that encodes time
      *
      * The ancient scribe and the COBOL programmer.
      * Separated by 1200 years. Running the same program.
      *================================================================
           EVALUATE WS-C-OPERAND
               WHEN "FISH  " DISPLAY "INSTRUCTION: SET THE NETS"
               WHEN "BIRD  " DISPLAY "INSTRUCTION: OBSERVE THE SKY"
               WHEN OTHER   DISPLAY "INSTRUCTION: " WS-C-OPERAND
           END-EVALUATE.
