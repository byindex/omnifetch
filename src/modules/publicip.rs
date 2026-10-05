use crate::module::{Module, ModuleOutput};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

pub struct PublicIp;

impl Module for PublicIp {
    fn name(&self) -> &'static str {
        "Public IP"
    }
    fn id(&self) -> &'static str {
        "publicip"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let ip = fetch_public_ip_udp_dns().or_else(fetch_public_ip_raw_http)?;

        let trimmed = ip.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(ModuleOutput::new("Public IP", trimmed))
        }
    }
}

fn build_dns_query(host: &str) -> Option<Vec<u8>> {
    let mut query = Vec::with_capacity(12 + host.len() + 6);
    query.extend_from_slice(&[
        0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    for part in host.split('.') {
        if part.is_empty() || part.len() > 63 {
            return None;
        }
        query.push(part.len() as u8);
        query.extend_from_slice(part.as_bytes());
    }
    query.push(0x00);
    query.extend_from_slice(&[0x00, 0x01, 0x00, 0x01]);
    Some(query)
}

/// Public IP via a UDP DNS query to OpenDNS anycast, skipping the TCP, TLS and HTTP handshakes.
fn fetch_public_ip_udp_dns() -> Option<String> {
    let host = &crate::config::get().network_cfg.publicip_host;
    let query = build_dns_query(host)?;

    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket
        .set_read_timeout(Some(Duration::from_millis(250)))
        .ok()?;
    socket
        .set_write_timeout(Some(Duration::from_millis(150)))
        .ok()?;

    let targets: [SocketAddr; 2] = [
        "208.67.222.222:53".parse().ok()?,
        "208.67.220.220:53".parse().ok()?,
    ];

    for target in &targets {
        if socket.send_to(&query, target).is_ok() {
            let mut buf = [0u8; 512];
            if let Ok((len, _)) = socket.recv_from(&mut buf)
                && let Some(ip) = parse_dns_a_record(&buf[..len])
            {
                return Some(ip);
            }
        }
    }
    None
}

pub fn parse_dns_a_record(data: &[u8]) -> Option<String> {
    if data.len() < 12 {
        return None;
    }
    // Transaction ID must match and RCODE (lower 4 bits of byte 3) must be 0
    if data[0] != 0x12 || data[1] != 0x34 || (data[3] & 0x0F) != 0 {
        return None;
    }

    let qdcount = u16::from_be_bytes([data[4], data[5]]) as usize;
    let ancount = u16::from_be_bytes([data[6], data[7]]) as usize;
    if ancount == 0 {
        return None;
    }

    let mut offset = 12;
    // Skip question section
    for _ in 0..qdcount {
        while offset < data.len() && data[offset] != 0 {
            offset += 1 + data[offset] as usize;
        }
        offset += 5; // 0 byte + 2 bytes QTYPE + 2 bytes QCLASS
    }

    // Parse answers
    for _ in 0..ancount {
        if offset >= data.len() {
            break;
        }
        // Name: pointer or label sequence
        if (data[offset] & 0xC0) == 0xC0 {
            offset += 2;
        } else {
            while offset < data.len() && data[offset] != 0 {
                offset += 1 + data[offset] as usize;
            }
            offset += 1;
        }
        if offset + 10 > data.len() {
            break;
        }
        let rtype = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 8; // skip rtype (2), rclass (2), ttl (4)
        let rdlength = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        if rtype == 1 && rdlength == 4 && offset + 4 <= data.len() {
            let b = &data[offset..offset + 4];
            return Some(format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3]));
        }
        offset += rdlength;
    }
    None
}

/// Plain TCP HTTP GET, used when curl is not installed.
fn fetch_public_ip_raw_http() -> Option<String> {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let fallback = &crate::config::get().network_cfg.publicip_fallback;
    let endpoints = [(fallback.as_str(), "/"), ("api.ipify.org", "/")];
    for (host, path) in endpoints {
        // The resolver will happily sit on a silent nameserver for 5 s.
        let addr = crate::sys::resolve_bounded(host, 80, Duration::from_millis(500))?;
        if let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(300)) {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(400)));
            let _ = stream.set_write_timeout(Some(Duration::from_millis(200)));

            let req = format!(
                "GET {path} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: omnifetch\r\nConnection: close\r\n\r\n"
            );
            if stream.write_all(req.as_bytes()).is_ok() {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 1024];
                while let Ok(n) = stream.read(&mut chunk) {
                    if n == 0 {
                        break;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.len() > 2048 {
                        break;
                    }
                }
                let text = String::from_utf8_lossy(&buf);
                if let Some(body) = text.split("\r\n\r\n").nth(1) {
                    let trimmed = body.trim().to_string();
                    if !trimmed.is_empty() && !trimmed.contains('<') {
                        return Some(trimmed);
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A DNS/UDP answer for `myip.opendns.com` pointing at 1.2.3.4.
    fn answer() -> Vec<u8> {
        vec![
            18, 52, 129, 128, 0, 1, 0, 1, 0, 0, 0, 0, 4, 109, 121, 105, 112, 7, 111, 112, 101, 110,
            100, 110, 115, 3, 99, 111, 109, 0, 0, 1, 0, 1, 192, 12, 0, 1, 0, 1, 0, 0, 0, 0, 4, 1,
            2, 3, 4,
        ]
    }

    #[test]
    fn dns_answer_is_rejected_on_wrong_id_or_rcode() {
        let mut wrong_id = answer();
        wrong_id[1] = 0x99;
        assert!(parse_dns_a_record(&wrong_id).is_none());

        let mut nxdomain = answer();
        nxdomain[3] = (nxdomain[3] & 0xF0) | 0x03;
        assert!(parse_dns_a_record(&nxdomain).is_none());

        let mut servfail = answer();
        servfail[3] = (servfail[3] & 0xF0) | 0x02;
        assert!(parse_dns_a_record(&servfail).is_none());
    }

    #[test]
    fn dns_answer_is_rejected_when_there_are_no_records() {
        let mut empty = answer();
        empty[7] = 0; // ANCOUNT = 0
        assert!(parse_dns_a_record(&empty).is_none());
    }

    #[test]
    fn dns_answer_of_zero_length_is_rejected() {
        assert!(parse_dns_a_record(&[]).is_none());
        assert!(parse_dns_a_record(&[0u8; 11]).is_none());
        assert!(parse_dns_a_record(&[0u8; 12]).is_none());
    }

    #[test]
    fn dns_answer_with_cname_chain_yields_the_last_address() {
        // Truncated to header plus question, then CNAME and A appended.
        let mut pkt = answer();
        pkt.truncate(34);
        pkt[6] = 0;
        pkt[7] = 2;
        // CNAME: pointer to the question name, type 5, ttl, then a one-label target.
        pkt.extend_from_slice(&[0xC0, 0x0C, 0, 5, 0, 1, 0, 0, 0, 0, 0, 3, 1, b'x', 0]);
        pkt.extend_from_slice(&[0xC0, 0x38, 0, 1, 0, 1, 0, 0, 0, 0, 0, 4, 5, 6, 7, 8]);
        assert_eq!(parse_dns_a_record(&pkt).as_deref(), Some("5.6.7.8"));
    }

    #[test]
    fn test_parse_dns_a_record() {
        let valid_packet = [
            18, 52, 129, 128, 0, 1, 0, 1, 0, 0, 0, 0, 4, 109, 121, 105, 112, 7, 111, 112, 101, 110,
            100, 110, 115, 3, 99, 111, 109, 0, 0, 1, 0, 1, 192, 12, 0, 1, 0, 1, 0, 0, 0, 0, 0, 4,
            1, 2, 3, 4,
        ];
        assert_eq!(
            parse_dns_a_record(&valid_packet),
            Some("1.2.3.4".to_string())
        );
    }

    #[test]
    fn test_parse_dns_truncated_does_not_panic() {
        assert_eq!(parse_dns_a_record(&[]), None);
        assert_eq!(parse_dns_a_record(&[18, 52]), None);
        assert_eq!(parse_dns_a_record(&[18, 52, 129, 128, 0, 1, 0, 1]), None);
    }

    #[test]
    fn test_build_dns_query() {
        let expected: &[u8] = b"\x12\x34\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x04myip\x07opendns\x03com\x00\x00\x01\x00\x01";
        assert_eq!(
            build_dns_query("myip.opendns.com").as_deref(),
            Some(expected)
        );
    }
}
