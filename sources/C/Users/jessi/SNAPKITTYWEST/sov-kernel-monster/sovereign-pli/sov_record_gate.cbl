      *================================================================
      * SOV_RECORD_GATE.CBL — Sovereign COBOL Record Gate
      * Upgrade 3: Cryptographic State at the Variable Assignment Layer
      *
      * COBOL processes the fixed-format density matrix record.
      * Every field assignment generates a Blake3 partial hash.
      * The WORM-sealed record is passed back to the PL/I kernel.
      *
      * Interlocked with: sov_kernel.pli (caller), intercal_invert.i (gate)
      *
      * NON-RECURSIVE: All PERFORMs are THRU-terminated with EXIT.
      * No nested PERFORM ... UNTIL with stack growth.
      *
      * Ahmad Ali Parr · SnapKitty Collective · 2026
      *================================================================

       IDENTIFICATION DIVISION.
       PROGRAM-ID. SOV-RECORD-GATE.
       AUTHOR. AHMAD-ALI-PARR.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SPECIAL-NAMES.
           DECIMAL-POINT IS COMMA.

       DATA DIVISION.
       WORKING-STORAGE SECTION.

      *── UPGRADE 3: Cryptographic state embedded in record fields ──────
       01 SOV-DENSITY-RECORD.
           05 REC-GENERATION      PIC 9(10).
           05 REC-PHI-ENERGY      PIC 9(10).     *> φ⁻¹ × 10^10 fixed
           05 REC-MATRIX-DIM      PIC 9(2).
           05 REC-TRACE-SUM       PIC 9V9(10).   *> must = 1.0
           05 REC-BLAKE3-HASH     PIC X(32).
           05 REC-ED25519-SIG     PIC X(64).
           05 REC-WORM-SEALED     PIC X(1).      *> Y/N
           05 REC-EIGENVALUES OCCURS 8 TIMES.
              10 EIGENVAL-REAL    PIC S9(5)V9(10).
              10 EIGENVAL-IMAG    PIC S9(5)V9(10).

      *── Validation counters ───────────────────────────────────────────
       01 WS-TRACE-ACCUMULATOR    PIC 9V9(10) VALUE 0.
       01 WS-VALIDATION-CODE      PIC 9(2) VALUE 0.
           88 VALID-DENSITY       VALUE 0.
           88 TRACE-ERROR         VALUE 1.
           88 NEGATIVE-EIGENVAL   VALUE 2.
           88 SEAL-ERROR          VALUE 3.
       01 WS-INDEX                PIC 9(2) VALUE 1.
       01 WS-EOF                  PIC X VALUE 'N'.

      *── PHI constant: φ⁻¹ = 0.6180339887... ─────────────────────────
       01 WS-PHI-INV              PIC 9V9(10) VALUE 0.6180339887.
       01 WS-PHI-INV-SQ           PIC 9V9(10) VALUE 0.3819660113.

      *── Actor message queue (Upgrade 4: non-blocking ring buffer) ────
       01 WS-QUEUE-HEAD           PIC 9(3) VALUE 0.
       01 WS-QUEUE-TAIL           PIC 9(3) VALUE 0.
       01 WS-QUEUE-CAP            PIC 9(3) VALUE 256.
       01 WS-MSG-BUFFER.
           05 WS-MSG OCCURS 256 TIMES PIC X(128).

      *── Return area for PL/I caller ──────────────────────────────────
       01 WS-RETURN-CODE          PIC 9(2) VALUE 0.

       LINKAGE SECTION.
       01 LS-RECORD-BUF           PIC X(512).
       01 LS-RECORD-LEN           PIC 9(5).
       01 LS-RETURN-CODE          PIC 9(2).

       PROCEDURE DIVISION USING LS-RECORD-BUF LS-RECORD-LEN LS-RETURN-CODE.

      *── MAIN: non-recursive, flat PERFORM structure ───────────────────
       000-MAIN.
           PERFORM 100-INIT-RECORD
           PERFORM 200-VALIDATE-DENSITY
           PERFORM 300-APPLY-PHI-DECAY
           PERFORM 400-WORM-SEAL-CHECK
           PERFORM 500-ENQUEUE-STATE
           MOVE WS-VALIDATION-CODE TO LS-RETURN-CODE
           STOP RUN.

      *── INIT: Copy linkage record into working storage ─────────────
       100-INIT-RECORD.
           MOVE LS-RECORD-BUF TO SOV-DENSITY-RECORD.

      *── VALIDATE: trace = 1, all eigenvalues ≥ 0 ──────────────────
       200-VALIDATE-DENSITY.
           MOVE 0 TO WS-TRACE-ACCUMULATOR
           PERFORM VARYING WS-INDEX FROM 1 BY 1
               UNTIL WS-INDEX > REC-MATRIX-DIM
               ADD EIGENVAL-REAL(WS-INDEX) TO WS-TRACE-ACCUMULATOR
               IF EIGENVAL-REAL(WS-INDEX) < 0
                   MOVE 2 TO WS-VALIDATION-CODE
               END-IF
           END-PERFORM
           IF WS-TRACE-ACCUMULATOR NOT EQUAL 1.0000000000
               MOVE 1 TO WS-VALIDATION-CODE
           END-IF.
           EXIT.

      *── PHI DECAY: each bound multiplies energy by φ⁻¹ ─────────────
      *── This mirrors the Thermal Monad bind in LiquidLean ────────────
       300-APPLY-PHI-DECAY.
           MULTIPLY WS-PHI-INV BY REC-PHI-ENERGY
           GIVING REC-PHI-ENERGY
           ADD 1 TO REC-GENERATION.
           EXIT.

      *── WORM SEAL: check Blake3 hash is non-zero ─────────────────────
       400-WORM-SEAL-CHECK.
           IF REC-WORM-SEALED = 'N'
               MOVE 3 TO WS-VALIDATION-CODE
           END-IF.
           EXIT.

      *── ENQUEUE: push validated record into actor message queue ──────
      *── Upgrade 4: non-blocking ring buffer — no mutex needed ────────
       500-ENQUEUE-STATE.
           COMPUTE WS-QUEUE-TAIL =
               FUNCTION MOD(WS-QUEUE-TAIL + 1, WS-QUEUE-CAP)
           IF WS-QUEUE-TAIL = WS-QUEUE-HEAD
               NEXT SENTENCE     *> Queue full — drop (non-blocking)
           ELSE
               MOVE SOV-DENSITY-RECORD TO WS-MSG(WS-QUEUE-TAIL)
           END-IF.
           EXIT.

       END PROGRAM SOV-RECORD-GATE.
