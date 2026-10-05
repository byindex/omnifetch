use std::ffi::CStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::module::{Module, ModuleOutput};

pub struct LocalIp;

impl Module for LocalIp {
    fn name(&self) -> &'static str {
        "Local IP"
    }
    fn id(&self) -> &'static str {
        "localip"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut values: Vec<String> = Vec::new();
        for iface in interfaces() {
            for ip in iface.ips {
                if !is_reportable(ip) {
                    continue;
                }
                let mac = iface
                    .mac
                    .as_deref()
                    .map(|m| format!(" ({m})"))
                    .unwrap_or_default();
                values.push(format!("{}: {ip}{mac}", iface.name));
            }
        }
        (!values.is_empty()).then(|| ModuleOutput::multi("Local IP", values))
    }
}

fn is_reportable(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => !(v4.is_loopback() || v4.is_link_local() || v4.is_unspecified()),
        IpAddr::V6(v6) => {
            let seg = v6.segments();
            !(v6.is_loopback() || v6.is_unspecified() || seg[0] == 0xfe80 || v6.is_multicast())
        }
    }
}

struct Interface {
    name: String,
    mac: Option<String>,
    ips: Vec<IpAddr>,
}

fn interfaces() -> Vec<Interface> {
    let mut head: *mut libc::ifaddrs = std::ptr::null_mut();
    if unsafe { libc::getifaddrs(&mut head) } != 0 {
        return Vec::new();
    }
    let mut v: Vec<Interface> = Vec::new();
    let mut cur = head;
    while !cur.is_null() {
        let ifa = unsafe { &*cur };
        cur = ifa.ifa_next;

        if ifa.ifa_name.is_null() || ifa.ifa_addr.is_null() {
            continue;
        }
        let name = unsafe { CStr::from_ptr(ifa.ifa_name) }
            .to_string_lossy()
            .into_owned();
        let mac = if ifa.ifa_addr as *const _ as usize == 0
            || unsafe { (*ifa.ifa_addr).sa_family } == libc::AF_PACKET as u16
        {
            read_mac(ifa.ifa_data)
        } else {
            None
        };

        let ip = socket_addr(ifa.ifa_addr).map(|(a, _)| a);

        match v.iter_mut().find(|i| i.name == name) {
            Some(existing) => {
                if existing.mac.is_none() {
                    existing.mac = mac;
                }
                if let Some(ip) = ip {
                    existing.ips.push(ip);
                }
            }
            None => v.push(Interface {
                name,
                mac,
                ips: ip.into_iter().collect(),
            }),
        }
    }
    unsafe { libc::freeifaddrs(head) };
    v
}

fn socket_addr(sa: *const libc::sockaddr) -> Option<(IpAddr, u16)> {
    if sa.is_null() {
        return None;
    }
    match unsafe { (*sa).sa_family } as i32 {
        libc::AF_INET => {
            let v4 = unsafe { *(sa as *const libc::sockaddr_in) };
            Some((
                IpAddr::V4(Ipv4Addr::from(u32::from_be(v4.sin_addr.s_addr))),
                u16::from_be(v4.sin_port),
            ))
        }
        libc::AF_INET6 => {
            let v6 = unsafe { *(sa as *const libc::sockaddr_in6) };
            Some((
                IpAddr::V6(Ipv6Addr::from(v6.sin6_addr.s6_addr)),
                u16::from_be(v6.sin6_port),
            ))
        }
        _ => None,
    }
}

fn read_mac(data: *mut libc::c_void) -> Option<String> {
    if data.is_null() {
        return None;
    }
    let addr = unsafe { *(data as *const libc::sockaddr_ll) }.sll_addr;
    if addr.iter().all(|&b| b == 0) {
        return None;
    }
    Some(
        addr.iter()
            .take(6)
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(":"),
    )
}
