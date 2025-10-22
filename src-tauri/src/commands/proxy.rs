use crate::dht::{DhtService, PrivacyMode};
use crate::AppState;
use tauri::Emitter;
use tauri::State;
// use tracing::info;
use libp2p::PeerId;
use std::net::Ipv4Addr;
use std::str::FromStr;
use tracing::{info, warn};

#[derive(Clone, serde::Serialize)]
pub struct ProxyNode {
    pub id: String,
    pub address: String,
    pub status: String,
    pub latency: u32,
    pub error: Option<String>,
}

/// Normalize user input into a TCP libp2p multiaddr (no WebSocket).
/// - Keeps `/p2p/<peerid>` suffix if present
/// - For numeric IPs -> `/ip4/...`
/// - For hostnames  -> `/dns4/...`
/// - Treats ws:// and wss:// as plain TCP (drops `/ws`)
pub fn normalize_to_multiaddr(input: &str) -> Result<String, String> {
    let s = input.trim();

    // If it's already a multiaddr, accept as-is.
    if s.starts_with('/') {
        return Ok(s.to_string());
    }

    // Extract optional /p2p/<peer-id> suffix if user pasted it after the url/host:port
    let (base, p2p_suffix) = if let Some((left, right)) = s.split_once("/p2p/") {
        (left, Some(right))
    } else {
        (s, None)
    };

    // Strip known schemes;
    let base = base
        .strip_prefix("ws://")
        .or_else(|| base.strip_prefix("wss://"))
        .or_else(|| base.strip_prefix("tcp://"))
        .unwrap_or(base);

    // Expect host:port
    let (host, port) = base
        .split_once(':')
        .ok_or_else(|| format!("invalid address; expected host:port (got: {input})"))?;

    // Decide ip4 vs dns4
    let proto = if Ipv4Addr::from_str(host).is_ok() {
        "ip4"
    } else {
        "dns4"
    };

    let mut m = format!("/{proto}/{host}/tcp/{port}");
    if let Some(pid) = p2p_suffix {
        // keep any additional path after /p2p/<peerid> (rare, but harmless)
        m.push_str("/p2p/");
        m.push_str(pid);
    }
    Ok(m)
}

#[tauri::command]
pub(crate) async fn proxy_connect(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    url: String,
    _token: String,
) -> Result<(), String> {
    info!("Connecting to proxy: {}", url);

    // 1) optimistic UI
    {
        let mut proxies = state.proxies.lock().await;
        if let Some(p) = proxies.iter_mut().find(|p| p.address == url) {
            p.status = "connecting".into();
            p.error = None;
            p.latency = 999;
            let _ = app.emit("proxy_status_update", p.clone());
        } else {
            // The ID should be the normalized multiaddr, but we don't have it yet.
            // We'll use the URL as a temporary ID and the event pump will fix it.
            let node = ProxyNode {
                id: url.clone(),
                address: url.clone(),
                status: "connecting".into(),
                latency: 999,
                error: None,
            };
            proxies.push(node.clone());
            let _ = app.emit("proxy_status_update", node);
        }
    }

    // 2) dial via DHT
    if let Some(dht) = state.dht.lock().await.as_ref() {
        let multi = normalize_to_multiaddr(&url)?;
        dht.connect_peer(multi).await?;
        Ok(())
    } else {
        Err("DHT not initialized".into())
    }
}

#[tauri::command]
pub(crate) async fn proxy_disconnect(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    url: String,
) -> Result<(), String> {
    info!("Disconnecting from proxy: {}", url);
    let maybe_peer_id = {
        let mut proxies = state.proxies.lock().await;
        proxies
            .iter_mut()
            .find(|p| p.address == url || p.id == url)
            .map(|p| {
                p.status = "offline".into();
                let _ = app.emit("proxy_status_update", p.clone());
                p.id.clone()
            })
    };

    if let Some(peer_id_str) = maybe_peer_id {
        if let Ok(peer_id) = PeerId::from_str(&peer_id_str) {
            if let Some(dht) = state.dht.lock().await.as_ref() {
                return dht.disconnect_peer(peer_id).await;
            }
        }
    }

    Err("Could not disconnect peer".into())
}

#[tauri::command]
pub(crate) async fn proxy_remove(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    url: String,
) -> Result<(), String> {
    info!("Removing proxy: {}", url);

    let maybe_peer_id = {
        let mut proxies = state.proxies.lock().await;
        let maybe_idx = proxies.iter().position(|p| p.address == url || p.id == url);
        if let Some(idx) = maybe_idx {
            let p = proxies.remove(idx);
            Some(p.id)
        } else {
            None
        }
    };

    if let Some(peer_id_str) = maybe_peer_id {
        if let Ok(peer_id) = PeerId::from_str(&peer_id_str) {
            if let Some(dht) = state.dht.lock().await.as_ref() {
                let _ = dht.disconnect_peer(peer_id).await;
            }
        }
    }

    let _ = app.emit("proxy_reset", ());
    Ok(())
}

#[tauri::command]
pub(crate) async fn list_proxies(state: State<'_, AppState>) -> Result<Vec<ProxyNode>, String> {
    let proxies = state.proxies.lock().await;
    Ok(proxies.clone())
}

#[tauri::command]
pub(crate) async fn proxy_echo(
    state: State<'_, AppState>,
    peer_id: String,
    payload: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let dht_guard = state.dht.lock().await;
    let dht: &DhtService = dht_guard
        .as_ref()
        .ok_or_else(|| "DHT not running".to_string())?;
    dht.echo(peer_id, payload).await
}

#[tauri::command]
pub(crate) async fn enable_privacy_routing(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    proxy_addresses: Vec<String>,
    mode: Option<String>,
) -> Result<(), String> {
    let privacy_mode = mode
        .as_deref()
        .map(PrivacyMode::from_str)
        .unwrap_or(PrivacyMode::Prefer);

    info!(
        "Enabling privacy routing through {} proxies (mode: {:?})",
        proxy_addresses.len(),
        privacy_mode
    );

    if proxy_addresses.is_empty() {
        return Err("No proxy addresses provided".into());
    }

    // Store the proxy addresses for routing and normalize to multiaddrs
    let mut normalized_proxies: Vec<String> = Vec::new();
    {
        let mut privacy_proxies = state.privacy_proxies.lock().await;
        privacy_proxies.clear();
        for addr in &proxy_addresses {
            match normalize_to_multiaddr(addr) {
                Ok(multiaddr) => {
                    privacy_proxies.push(multiaddr.clone());
                    normalized_proxies.push(multiaddr);
                    info!("Added proxy for privacy routing: {}", addr);
                }
                Err(e) => {
                    warn!("Failed to normalize proxy address {}: {}", addr, e);
                }
            }
        }
    }

    if normalized_proxies.is_empty() {
        return Err("No valid proxy addresses provided".into());
    }

    // Enable privacy routing in DHT service
    if let Some(dht) = state.dht.lock().await.as_ref() {
        dht.update_privacy_proxy_targets(normalized_proxies.clone())
            .await?;
        dht.enable_privacy_routing(privacy_mode).await?;
    } else {
        return Err("DHT not initialized".into());
    }

    let _ = app.emit("privacy_routing_enabled", normalized_proxies.len());
    Ok(())
}

#[tauri::command]
pub(crate) async fn disable_privacy_routing(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Disabling privacy routing");

    // Clear stored proxy addresses
    {
        let mut privacy_proxies = state.privacy_proxies.lock().await;
        privacy_proxies.clear();
    }

    // Disable privacy routing in DHT service
    if let Some(dht) = state.dht.lock().await.as_ref() {
        dht.update_privacy_proxy_targets(Vec::new()).await?;
        dht.disable_privacy_routing().await?;
    } else {
        return Err("DHT not initialized".into());
    }

    let _ = app.emit("privacy_routing_disabled", ());
    Ok(())
}
