*> =====================================================================
*> COPYBOOK : COBUNIT
*> PURPOSE : Pure-COBOL unit test assertion library
*> USAGE : copy COBUNIT in working-storage / procedure
*> =====================================================================
*> ---- STATE -------------------------------------------------------
01 cu-state.
05 cu-total pic 9(5) value zero.
05 cu-passed pic 9(5) value zero.
05 cu-failed pic 9(5) value zero.
05 cu-current pic x(60) value spaces.
05 cu-failure-code pic 9(4) value zero.
01 cu-msg pic x(78).
01 cu-num-edited pic -(9)9.99.
01 cu-num-edited-2 pic -(9)9.99.
