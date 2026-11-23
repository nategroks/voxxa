//! Global outbound-network counter — the data behind the privacy UI.
//!
//! Every module that initiates an outbound HTTP request — the presenter drivers,
//! Planning Center client, discovery probes, Whisper model downloader — calls
//! [`record_request`] immediately before `Client::send`. The frontend polls
//! [`get_network_stats`] and renders the count alongside Voxxa's promise that
//! the only outbound traffic comes from things the user explicitly connected to.
//!
//! The update check (`updater::find`, ADR-020) counts as one request: the
//! plugin's own client also fetches the update itself when the operator
//! confirms an install.

use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

static OUTBOUND_REQUESTS: AtomicU64 = AtomicU64::new(0);

/// Bump the counter. Cheap (atomic add) and lock-free — safe to call from any
/// async context including spawn_blocking threads.
pub fn record_request() {
    OUTBOUND_REQUESTS.fetch_add(1, Ordering::Relaxed);
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkStats {
    pub outbound_requests: u64,
}

pub fn snapshot() -> NetworkStats {
    NetworkStats {
        outbound_requests: OUTBOUND_REQUESTS.load(Ordering::Relaxed),
    }
}
