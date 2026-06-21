// TSL UMD 3.1 tally monitor.
//
// Listens for TSL 3.1 UMD datagrams over UDP and exposes the latest tally state
// and label per display address — the broadcast-standard way a vision mixer or
// router distributes tally. This is a passive diagnostic: it only ever receives.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use tokio::sync::{oneshot, RwLock};

/// One TSL 3.1 display message (18 bytes on the wire: 1 address + 1 control +
/// 16 ASCII label).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TslDisplay {
    /// Display address (0–126)
    pub address: u8,
    /// Tally lamps 1–4 (commonly 1 = programme/red, 2 = preview/green)
    pub tally: [bool; 4],
    /// Brightness 0–3
    pub brightness: u8,
    /// 16-character label, trailing padding trimmed
    pub label: String,
}

/// Parse a buffer of one or more 18-byte TSL 3.1 UMD messages.
pub fn parse_tsl31(data: &[u8]) -> Vec<TslDisplay> {
    data.chunks_exact(18)
        .map(|chunk| {
            let control = chunk[1];
            TslDisplay {
                // The top bit (0x80) is the header marker; the address is the
                // low 7 bits.
                address: chunk[0] & 0x7F,
                tally: [
                    control & 0x01 != 0,
                    control & 0x02 != 0,
                    control & 0x04 != 0,
                    control & 0x08 != 0,
                ],
                brightness: (control >> 4) & 0x03,
                label: String::from_utf8_lossy(&chunk[2..18])
                    .trim_end_matches([' ', '\0'])
                    .to_string(),
            }
        })
        .collect()
}

/// Shared listener state.
#[derive(Default)]
pub struct TslMonitorInner {
    port: Option<u16>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    displays: HashMap<u8, TslDisplay>,
}

pub type TslState = Arc<RwLock<TslMonitorInner>>;

pub fn create_tsl_state() -> TslState {
    Arc::new(RwLock::new(TslMonitorInner::default()))
}

/// Start the TSL UMD listener on `port` (default 8900). Returns the bound port.
pub async fn start_monitor(state: TslState, port: Option<u16>) -> Result<u16, String> {
    {
        let inner = state.read().await;
        if inner.port.is_some() {
            return Err("TSL monitor is already running".to_string());
        }
    }

    let port = port.unwrap_or(8900);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let socket = UdpSocket::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind UDP port {}: {}", port, e))?;
    let bound = socket.local_addr().map_err(|e| e.to_string())?.port();

    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
    let task_state = state.clone();

    tokio::spawn(async move {
        let mut buf = [0u8; 2048];
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                recv = socket.recv_from(&mut buf) => {
                    if let Ok((len, _src)) = recv {
                        let displays = parse_tsl31(&buf[..len]);
                        if !displays.is_empty() {
                            let mut inner = task_state.write().await;
                            for display in displays {
                                inner.displays.insert(display.address, display);
                            }
                        }
                    }
                }
            }
        }
    });

    let mut inner = state.write().await;
    inner.port = Some(bound);
    inner.shutdown_tx = Some(shutdown_tx);
    Ok(bound)
}

/// Stop the listener.
pub async fn stop_monitor(state: TslState) -> Result<(), String> {
    let mut inner = state.write().await;
    if let Some(tx) = inner.shutdown_tx.take() {
        let _ = tx.send(());
        inner.port = None;
        Ok(())
    } else {
        Err("TSL monitor is not running".to_string())
    }
}

/// The bound port, or None if not running.
pub async fn monitor_port(state: TslState) -> Option<u16> {
    state.read().await.port
}

/// A snapshot of the current displays, sorted by address.
pub async fn snapshot(state: TslState) -> Vec<TslDisplay> {
    let inner = state.read().await;
    let mut displays: Vec<TslDisplay> = inner.displays.values().cloned().collect();
    displays.sort_by_key(|d| d.address);
    displays
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(address: u8, control: u8, label: &str) -> Vec<u8> {
        let mut msg = vec![0x80 | address, control];
        let mut bytes = label.as_bytes().to_vec();
        bytes.resize(16, b' ');
        msg.extend_from_slice(&bytes);
        msg
    }

    #[test]
    fn parses_tsl31_message() {
        // Address 3, tally 1 + 3 on, brightness 3, label "CAM 3".
        let msg = message(3, 0x01 | 0x04 | (3 << 4), "CAM 3");
        assert_eq!(msg.len(), 18);

        let displays = parse_tsl31(&msg);
        assert_eq!(displays.len(), 1);
        let d = &displays[0];
        assert_eq!(d.address, 3);
        assert_eq!(d.tally, [true, false, true, false]);
        assert_eq!(d.brightness, 3);
        assert_eq!(d.label, "CAM 3");
    }

    #[test]
    fn listener_receives_and_parses_over_udp() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let state = create_tsl_state();
            // Bind an ephemeral port so the test never clashes.
            let port = start_monitor(state.clone(), Some(0)).await.unwrap();

            let sender = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let msg = message(5, 0x02, "PGM"); // address 5, tally 2 (preview)
            sender.send_to(&msg, ("127.0.0.1", port)).await.unwrap();

            // Give the listener task a moment to receive and record it.
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            let snap = snapshot(state.clone()).await;
            stop_monitor(state).await.unwrap();

            let found = snap
                .iter()
                .find(|d| d.address == 5)
                .expect("display 5 should have been recorded");
            assert_eq!(found.tally, [false, true, false, false]);
            assert_eq!(found.label, "PGM");
        });
    }
}
