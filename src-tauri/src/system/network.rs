// Network address detection
// Resolves the host's primary IPv4 — the address a remote operator points their
// client at — so the GUI can show "connect to <ip>:5250" rather than leaving the
// operator to guess which interface to use.

use std::net::{IpAddr, UdpSocket};

/// Best-effort primary IPv4 of this host: the address the OS would use to reach
/// the wider network. Implemented by opening a UDP socket and "connecting" it to
/// a public address — no packets are sent; the kernel just resolves which local
/// interface would route there, and we read that interface's address back. This
/// naturally yields the real LAN address rather than loopback.
///
/// Loopback, link-local (169.254/16) and Tailscale's CGNAT range (100.64/10) are
/// rejected so an overlay-network address never masquerades as the primary one.
pub fn primary_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    // A routable public address; nothing is transmitted by connect() for UDP.
    socket.connect("8.8.8.8:80").ok()?;
    let ip = socket.local_addr().ok()?.ip();

    match ip {
        IpAddr::V4(v4) => {
            let [a, b, _, _] = v4.octets();
            // Tailscale and other CGNAT addresses live in 100.64.0.0/10.
            let is_cgnat = a == 100 && (0x40..=0x7F).contains(&b);
            if v4.is_loopback() || v4.is_link_local() || is_cgnat {
                None
            } else {
                Some(v4.to_string())
            }
        }
        // The GUI shows an IPv4 endpoint; an IPv6-only result is not useful here.
        IpAddr::V6(_) => None,
    }
}
