      * SEB Event Copybook (RPG)
      * Generated from: SEB_SOVEREIGN_EVENT_BUS_MASTER_SPECIFICATION.xml
      * Version: 1.0.0
      * Layer: L4 - Mainframe Integration
      * Platform: IBM i (RPG/ILE)
      *
      * This copybook defines the event envelope structure for SEB events
      * exchanged between the Sovereign Event Bus and IBM i systems.
      *
      * Total overhead: 196 bytes (68 byte header + 128 byte footer)
      * Payload: variable length (stored in BLOB_FILE)
      *
      * Cryptography: Blake3 (32 bytes) + Ed25519 (64 bytes)
      * Encoding: UTF-8 for all character fields

      * ================================================================
      * HEADER SECTION (68 bytes)
      * ================================================================

      * Offset in WORM chain (8 bytes)
      D  SEB_OFFSET             I    0   0       8,0 VALUE(0)

      * Timestamp (ISO-8601, 26 bytes)
      * Format: YYYY-MM-DDTHH:MM:SS.sssZ
      D  SEB_TIMESTAMP          C    1   26A

      * Agent ID (16 bytes, left-justified, blank-padded)
      * Examples: "FISCAL_SETTLE", "BIFROST", etc.
      D  SEB_AGENT_ID           C    27  42A

      * Event Type (10 bytes, left-justified, blank-padded)
      * Examples: "SETTLEMENT", "ROUTING", "VERIFY", etc.
      D  SEB_EVENT_TYPE         C    43  52A

      * Payload Size in bytes (4 bytes, binary)
      * Range: 0 to 2GB (via variable-length file)
      D  SEB_PAYLOAD_SIZE       I    53  56       4,0 VALUE(0)

      * Reserved for future use (12 bytes)
      D  SEB_RESERVED           C    57  68A


      * ================================================================
      * FOOTER SECTION (128 bytes - Cryptographic Seal)
      * ================================================================

      * Previous event hash (Blake3, 64 hex chars = 32 bytes stored)
      * Used for chain validation
      D  SEB_PREV_HASH          C    69  100A

      * Event hash (Blake3, 64 hex chars)
      * Hash of: header + payload + prev_hash
      D  SEB_EVENT_HASH         C    101 132A

      * Ed25519 signature (128 hex chars = 64 bytes)
      * Signs: event_hash with agent's private key
      D  SEB_SIGNATURE          C    133 196A


      * ================================================================
      * VARIABLE PAYLOAD (stored separately in BLOB_FILE)
      * ================================================================
      *
      * Payload structure (JSON format, UTF-8 encoded):
      * {
      *   "intent": { ... },
      *   "context": { ... },
      *   "authority": { ... },
      *   "continuation": { ... },
      *   "evidence": [ ... ]
      * }
      *
      * Payload is stored in external file:
      * - Filename: SEB_PAYLOAD_<OFFSET>_<AGENT_ID>.blob
      * - Maximum size: 2GB (4-byte offset field limitation)
      * - Encoding: UTF-8
      * - Access: Random (seekable)


      * ================================================================
      * ENVELOPE DEFINITION - Data structure for RPC
      * ================================================================

      D SEBEVENT            DS
      D  sb_offset                 1      8I 0
      D  sb_timestamp             9     34A
      D  sb_agent_id              35    50A
      D  sb_event_type            51    60A
      D  sb_payload_size          61    64I 0
      D  sb_reserved              65    76A
      D  sb_prev_hash             77   108A
      D  sb_event_hash           109   140A
      D  sb_signature            141   204A


      * ================================================================
      * DERIVED FIELDS (computed by adapter)
      * ================================================================

      D SEB_ENVELOPE_SIZE      C      L'SEBEVENT

      * Constants for validation
      D SEB_MAX_PAYLOAD_SIZE   C      2147483647        * 2^31 - 1
      D SEB_HASH_LENGTH        C      64                * Blake3 hex
      D SEB_SIG_LENGTH         C      128               * Ed25519 hex
      D SEB_TIMESTAMP_FORMAT   C      'YYYY-MM-DDTHH:MM:SS.sssZ'


      * ================================================================
      * ERROR CODES
      * ================================================================

      D SEB_ERR_SUCCESS        C      0
      D SEB_ERR_INVALID_OFFSET C      1
      D SEB_ERR_INVALID_SIZE   C      2
      D SEB_ERR_HASH_MISMATCH  C      3
      D SEB_ERR_SIG_INVALID    C      4
      D SEB_ERR_FILE_READ      C      5
      D SEB_ERR_FILE_WRITE     C      6
      D SEB_ERR_CHAIN_BROKEN   C      7
      D SEB_ERR_PAYLOAD_CORRUPT C      8


      * ================================================================
      * PROCEDURE PROTOTYPES (called by SOVEREIGN_LEDGER)
      * ================================================================

      * Append event to SEB chain
      D SEB_Append_Event    PR                  EXTPGM('SEB_APPEND')
      D  pi_envelope                      DS    QUALIFIED
      D  pi_payload                             VARYING
      D  po_offset                        10I 0
      D  po_error_code                    10I 0

      * Verify event chain integrity
      D SEB_Verify_Chain    PR                  EXTPGM('SEB_VERIFY')
      D  pi_start_offset                  10I 0
      D  pi_end_offset                    10I 0
      D  po_chain_valid                    1
      D  po_error_code                    10I 0

      * Read event by offset
      D SEB_Read_Event      PR                  EXTPGM('SEB_READ')
      D  pi_offset                        10I 0
      D  po_envelope                      DS    QUALIFIED
      D  po_payload                     32767 VARYING
      D  po_error_code                    10I 0

      * Commit offset marker (idempotency key)
      D SEB_Commit_Offset   PR                  EXTPGM('SEB_COMMIT')
      D  pi_bifrost_hash                 128A
      D  pi_offset                        10I 0
      D  po_committed                     1
      D  po_error_code                    10I 0


      * ================================================================
      * WORM CHAIN OPERATIONS
      * ================================================================

      * The SEB chain is immutable: once written, events cannot be modified
      * Each append:
      * 1. Computes Blake3 hash of (header + payload + prev_hash)
      * 2. Signs hash with agent's Ed25519 private key
      * 3. Writes header + footer to SEB_CHAIN_LOG file
      * 4. Writes payload to SEB_PAYLOAD_<offset>.blob file
      * 5. Returns offset for SOVEREIGN_LEDGER idempotency tracking
      *
      * Bifrost_Hash deduplication:
      * - If settlement already exists in SOVEREIGN_LEDGER with same
      *   Bifrost_Hash, SEB_Append_Event returns existing offset
      * - No duplicate settlements are possible
      * - Provides ACID compliance for distributed transactions

