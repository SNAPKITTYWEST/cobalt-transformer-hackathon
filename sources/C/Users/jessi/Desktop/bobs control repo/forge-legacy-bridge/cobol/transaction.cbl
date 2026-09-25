       IDENTIFICATION DIVISION.
      std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub soul_id:      u64,
    pub timestamp:    u64,   // unix seconds at hydration time
    pub chain_height: u64,
    pub loaded_funcs: Vec<String>,
    pub last_result:  Option<i64>,
    pub custom:       serde_json::Value,
}

impl ContextSnapshot {
    pub fn new(soul_id: u64) -> Self {
        Self {
            soul_id,
            timestamp:    now_secs(),
            chain_height: 0,
            loaded_funcs: Vec::new(),
            last_result:  None,
            custom:       serde_json::Value::Null,
        }
    }
}

#[derive(Clone, Default)]
pub struct ContextHydrator {
    snapshots: Arc<Mutex<HashMap<u64, ContextSnapshot>>>,
}

impl ContextHydrator {
    pub fn new() -> Self { Self::default() }

    pub fn hydrate(&self, soul_id: u64, snap: ContextSnapshot) {
        if let Ok(mut m) = self.snapshots.lock() {
            m.insert(soul_id, snap);
        }
    }

    pub fn get(&self, soul_id: u64) -> Option<ContextSnapshot> {
        self.snapshots.lock().ok()?.get(&soul_id).cloned()
    }

    /// Update a single key inside the `custom` JSON blob.
    pub fn freshen(&self, soul_id: u64, key: &str, value: serde_json::Value) {
        if let Ok(mut m) = self.snapshots.lock() {
            if let Some(snap) = m.get_mut(&soul_id) {
                snap.timestamp = now_secs();
                match &mut snap.custom {
                    serde_json::Value::Object(map) => { map.insert(key.to_string(), value); }
                    other => {
                        let mut map = serde_json::Map::new();
                        map.insert(key.to_string(), value);
                        *other = serde_json::Value::Object(map);
                    }
                }
            }
        }
    }

    pub fn age_seconds(&self, soul_id: u64) -> Option<u64> {
        let snap = self.get(soul_id)?;
        Some(now_secs().saturating_sub(snap.timestamp))
    }

    pub fn is_stale(&self, soul_id: u64, max_age_secs: u64) -> bool {
        self.age_seconds(soul_id)
            .map(|age| age >= max_age_secs)
            .unwrap_or(true) // unknown soul = stale
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(soul_id: u64) -> ContextSnapshot {
        ContextSnapshot {
            soul_id,
            timestamp:    now_secs(),
            chain_height: 5,
            loaded_funcs: vec!["emoji_fn_abc".into()],
            last_result:  Some(42),
            custom:       serde_json::json!({ "role": "architect" }),
        }
    }

    #[test]
    fn hydrate_and_retrieve() {
        let h = ContextHydrator::new();
        h.hydrate(1, snap(1));ES
               MOVE 'INVALID' TO TX-STATUS
               MOVE 'N' TO WS-VALID-FLAG
           ELSE IF TX-AMOUNT <= WS-THRESHOLD
               MOVE 'INVALID' TO TX-STATUS
               MOVE 'N' TO WS-VALID-FLAG
           ELSE
               MOVE 'APPROVED' TO TX-STATUS
               MOVE 'Y' TO WS-VALID-FLAG
           END-IF.

      *----------------------------------------------------------------
      * 300 — SUBLEQ gate: A=amount, B=threshold, C=seal or reject
      *       If A > B: route to Rust seal (C fires)
      *       If A <= B: reject (set status REJECTED)
      *----------------------------------------------------------------
       300-SUBLEQ-GATE.
           IF WS-VALID-FLAG = 'N'
               MOVE 'REJECTED' TO TX-STATUS
           END-IF.

      *----------------------------------------------------------------
      * 400 — Output JSON for Rust FFI consumption
      *       Rust parses this output and routes to WORM chain
      *----------------------------------------------------------------
       400-OUTPUT-JSON.
           MOVE TX-AMOUNT TO WS-AMOUNT-DISPLAY
           DISPLAY '{'
               '"id":"'      FUNCTION TRIM(TX-ID)       '",'
               '"amount":'   FUNCTION TRIM(ENV-TX-AMOUNT) ','
               '"source":"'  FUNCTION TRIM(TX-SOURCE)    '",'
               '"dest":"'    FUNCTION TRIM(TX-DEST)      '",'
               '"category":"' FUNCTION TRIM(TX-CATEGORY) '",'
               '"timestamp":"' FUNCTION TRIM(TX-TIMESTAMP) '",'
               '"status":"'  TX-STATUS                   '",'
               '"bridge":"cobol-1959"'
           '}'
           .
