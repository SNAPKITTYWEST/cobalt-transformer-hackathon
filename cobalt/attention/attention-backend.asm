         TITLE 'ATTN-GEMM - z/Architecture HLASM attention GEMM backend'
* =====================================================================
* attention-backend.asm — optional optimized backend for the COBOL
* reference kernel (attention.cob, ATTN-CALC).
*
* Implements the IDENTICAL mathematical operation:
*
* SCORE[I,J] = SUM(K=1..64) Q[I,K] * K[J,K] / 8
*
* (Q x K^T, then scale by 1/sqrt(d_k) = 1/8), seq len = head dim = 64.
*
* Target: IBM z/Architecture, HLASM, AMODE 31, IBM Enterprise COBOL
* callers. Enterprise COBOL USAGE COMP-2 on z/OS is long HEXADECIMAL
* floating point, so this routine uses HFP long instructions (LD/MD/
* ADR/DD/STD). It must NOT be linked against GnuCOBOL, whose COMP-2
* is IEEE binary64 — the encodings are incompatible. On GnuCOBOL keep
* ATTN-USE-BACKEND = 'N' and the pure-COBOL kernel runs alone.
*
* OS linkage: static/dynamic CALL from COBOL passes in GPR1 a pointer
* to a fullword address list of the USING operands:
* +0 -> Q-VAL (64x64 COMP-2, row-major)
* +4 -> K-VAL (64x64 COMP-2)
* +8 -> SCORE (64x64 COMP-2)
* The routine returns GPR15 = 0 on success (COBOL ON EXCEPTION sees a
* nonzero/abend only if the entry is missing, so the reference kernel
* remains the sole mandatory implementation).
*
* Loop structure (semantics-preserving optimizations only):
* - SCORE is walked CONTIGUOUSLY (one pointer, +8 per element): the
* 64x64 output is visited linearly, no per-element subscript math.
* - Q row base is held in a register across the inner k loop; K row
* base likewise — only the k*8 byte offset is incremented.
* - The accumulator lives in FPR0 across the whole k loop; SCORE is
* written once per (i,j), not read-modify-written 64 times. This
* changes NOTHING numerically: FPR0 is HFP long, identical to the
* storage format, so accumulation order and rounding are the same
* as the COBOL COMPUTE chain.
* - Scaling happens ONCE per element, after the loop (never inside).
* =====================================================================
ATTNGEMM CSECT
ATTNGEMM AMODE 31
         SAVE (14,12) save caller's registers
         LR R12,R15 establish base
         USING ATTNGEMM,R12
         L R2,0(,R1) R2 = Q base
         L R3,4(,R1) R3 = K base
         L R4,8(,R1) R4 = SCORE base (walked linearly)
         LR R7,R4 R7 = current SCORE element addr
         LR R5,R2 R5 = Q row base (i)
         LA R10,64 R10 = i counter (rows of Q)

* -------- outer loop over I = 0..63 --------------------------------
ILOOP LR R6,R3 R6 = K row base reset (j)
         LA R11,64 R11 = j counter (rows of K)

* -------- middle loop over J = 0..63 --------------------------------
JLOOP SR R9,R9 R9 = k*8 byte offset = 0
         LA R0,64 R0 = k counter
         SDR F0,F0 F0 = accumulator = +0.0

* -------- inner contraction over K = 0..63 ---------------------------
* F0 += Q(i,k) * K(j,k) both addressed via R9 offset
KLOOP LD F1,0(R9,R5) F1 = Q(I,K)
         MD F1,0(R9,R6) F1 *= K(J,K) (HFP long mult)
         ADR F0,F1 F0 = F0 + F1
         LA R9,8(R9) advance k*8 offset
         BCT R0,KLOOP next k

* -------- scale by 1/sqrt(d_k) = 1/8, then store ---------------------
         DD F0,=D'8.0' F0 /= 8 (after loop only)
         STD F0,0(,R7) SCORE(I,J) = F0

         LA R7,8(R7) next SCORE element (contiguous)
         LA R6,512(R6) next K row (64 * 8 bytes)
         BCT R11,JLOOP next j

         LA R5,512(R5) next Q row (64 * 8 bytes)
         BCT R10,ILOOP next i

         SR R15,R15 return code 0
         RETURN (14,12),RC=(15) restore and go back
         LTORG
         DROP R12
         END ATTNGEMM
