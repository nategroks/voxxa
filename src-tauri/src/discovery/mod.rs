//! Auto-discovery of presentation apps on the local machine and LAN.
//!
//! Two strategies in parallel:
//!   - localhost port probes against the known defaults of every API-capable
//!     driver (ProPresenter 7.9+ REST, FreeShow, OpenLP v2)
//!   - mDNS browsing for ProPresenter's `_pro7stagedsply._tcp.local.` service
//!     announcement, which gives us host candidates for REST probing on 1025
//!
//! Total wall-time is bounded by `timeout`; localhost probes finish in
//! milliseconds, mDNS browse runs to the budget.

use crate::presenters::PresenterKind;
use serde::Serialize;
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoverySource {
    Localhost,
    Mdns,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredService {
    pub kind: PresenterKind,
    pub host: String,
    pub port: u16,
    /// Display string for the picker UI.
    pub name: String,
    pub source: DiscoverySource,
}

/// Run all discovery strategies in parallel and return the union of results.
pub async fn discover_all(timeout: Duration) -> Vec<DiscoveredService> {
    let (mut local, mut mdns) = tokio::join!(probe_localhost(), browse_mdns(timeout));
    let mut all = Vec::with_capacity(local.len() + mdns.len());
    all.append(&mut local);
    all.append(&mut mdns);

    // Dedupe on (kind, host, port) — a host might surface via both localhost
    // probing and mDNS if the user is running everything on one machine.
    all.sort_by(|a, b| {
        (a.kind as u8, &a.host, a.port).cmp(&(b.kind as u8, &b.host, b.port))
    });
    all.dedup_by(|a, b| a.kind == b.kind && a.host == b.host && a.port == b.port);
    all
}

async fn probe_localhost() -> Vec<DiscoveredService> {
    let (pro7, freeshow, openlp, opensong) = tokio::join!(
        probe_pro7_rest("127.0.0.1", 1025),
        probe_freeshow("127.0.0.1", 5506),
        probe_openlp("127.0.0.1", 4316),
        probe_opensong("127.0.0.1", 8082),
    );
    [pro7, freeshow, openlp, opensong]
        .into_iter()
        .flatten()
        .collect()
}

async fn probe_opensong(host: &str, port: u16) -> Option<DiscoveredService> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .ok()?;
    let url = format!("http://{host}:{port}/presentation/status");
    crate::net_stats::record_request();
    let res = client.get(&url).send().await.ok()?;
    let status = res.status();
    // OpenSong returns XML on /status; 401 means we found it but auth is set.
    if !status.is_success()
        && status != reqwest::StatusCode::UNAUTHORIZED
        && status != reqwest::StatusCode::NOT_FOUND
    {
        return None;
    }
    Some(DiscoveredService {
        kind: PresenterKind::OpenSong,
        host: host.to_string(),
        port,
        name: format!("OpenSong on {host}"),
        source: DiscoverySource::Localhost,
    })
}

async fn probe_pro7_rest(host: &str, port: u16) -> Option<DiscoveredService> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .ok()?;
    let url = format!("http://{host}:{port}/v1/version");
    crate::net_stats::record_request();
    let res = client.get(&url).send().await.ok()?;
    if !res.status().is_success() {
        return None;
    }
    let body = res.text().await.ok()?;
    // Reject services that happen to answer on the port but aren't ProPresenter.
    if !body.contains("name") && !body.contains("version") {
        return None;
    }
    Some(DiscoveredService {
        kind: PresenterKind::ProPresenter7Rest,
        host: host.to_string(),
        port,
        name: format!("ProPresenter on {host}"),
        source: DiscoverySource::Localhost,
    })
}

async fn probe_freeshow(host: &str, port: u16) -> Option<DiscoveredService> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .ok()?;
    let url = format!("http://{host}:{port}");
    crate::net_stats::record_request();
    let res = client
        .post(&url)
        .json(&serde_json::json!({ "action": "get_shows" }))
        .send()
        .await
        .ok()?;
    if !res.status().is_success() {
        return None;
    }
    Some(DiscoveredService {
        kind: PresenterKind::FreeShow,
        host: host.to_string(),
        port,
        name: format!("FreeShow on {host}"),
        source: DiscoverySource::Localhost,
    })
}

async fn probe_openlp(host: &str, port: u16) -> Option<DiscoveredService> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .ok()?;
    let url = format!("http://{host}:{port}/api/v2/core/poll");
    crate::net_stats::record_request();
    let res = client.get(&url).send().await.ok()?;
    // 401 still means OpenLP is there — just protected by Basic Auth.
    if !res.status().is_success() && res.status() != reqwest::StatusCode::UNAUTHORIZED {
        return None;
    }
    Some(DiscoveredService {
        kind: PresenterKind::OpenLpV2,
        host: host.to_string(),
        port,
        name: format!("OpenLP on {host}"),
        source: DiscoverySource::Localhost,
    })
}

/// mDNS browse for ProPresenter. The crate's receiver is sync, so we run on a
/// blocking task with a hard deadline.
async fn browse_mdns(timeout: Duration) -> Vec<DiscoveredService> {
    tokio::task::spawn_blocking(move || browse_mdns_blocking(timeout))
        .await
        .unwrap_or_default()
}

fn browse_mdns_blocking(timeout: Duration) -> Vec<DiscoveredService> {
    let mdns = match mdns_sd::ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => {
            log::warn!("[DISCOVERY] mDNS daemon unavailable: {e}");
            return vec![];
        }
    };

    let service_type = "_pro7stagedsply._tcp.local.";
    let receiver = match mdns.browse(service_type) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("[DISCOVERY] mDNS browse failed: {e}");
            let _ = mdns.shutdown();
            return vec![];
        }
    };

    let deadline = Instant::now() + timeout;
    let mut seen: HashSet<(String, u16)> = HashSet::new();
    let mut out: Vec<DiscoveredService> = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match receiver.recv_timeout(remaining) {
            Ok(mdns_sd::ServiceEvent::ServiceResolved(info)) => {
                let hostname = info.get_hostname().trim_end_matches('.').to_string();
                for addr in info.get_addresses() {
                    let host = addr.to_string();
                    // ProPresenter announces the stage-display port via mDNS; the REST
                    // API runs on a separate user-configurable port. The default is
                    // 1025, which we surface here. If the user has changed it, the
                    // Test step in Settings will fail loud and they can correct.
                    let port = 1025u16;
                    if !seen.insert((host.clone(), port)) {
                        continue;
                    }
                    out.push(DiscoveredService {
                        kind: PresenterKind::ProPresenter7Rest,
                        host,
                        port,
                        name: format!("ProPresenter on {hostname}"),
                        source: DiscoverySource::Mdns,
                    });
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
    let _ = mdns.shutdown();
    out
}
