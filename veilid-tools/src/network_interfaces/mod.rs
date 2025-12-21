mod tools;

use crate::*;
use serde::*;

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "linux", target_os = "android"))] {
        mod netlink;
        use self::netlink::PlatformSupportNetlink as PlatformSupport;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
        mod sockaddr_tools;
        use self::windows::PlatformSupportWindows as PlatformSupport;
    } else if #[cfg(any(target_os = "macos", target_os = "ios"))] {
        mod apple;
        mod sockaddr_tools;
        use self::apple::PlatformSupportApple as PlatformSupport;
    } else if #[cfg(target_os = "openbsd")] {
        mod openbsd;
        mod sockaddr_tools;
        use self::openbsd::PlatformSupportOpenBSD as PlatformSupport;
    } else if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {
        mod wasm;
        use self::wasm::PlatformSupportWasm as PlatformSupport;
    } else {
        compile_error!("No network interfaces support for this platform!");
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Clone, Serialize, Deserialize)]
pub enum IfAddr {
    V4(Ifv4Addr),
    V6(Ifv6Addr),
}

impl IfAddr {
    #[must_use]
    pub fn ip(&self) -> IpAddr {
        match *self {
            IfAddr::V4(ref ifv4_addr) => IpAddr::V4(ifv4_addr.ip),
            IfAddr::V6(ref ifv6_addr) => IpAddr::V6(ifv6_addr.ip),
        }
    }
    #[must_use]
    pub fn netmask(&self) -> IpAddr {
        match *self {
            IfAddr::V4(ref ifv4_addr) => IpAddr::V4(ifv4_addr.netmask),
            IfAddr::V6(ref ifv6_addr) => IpAddr::V6(ifv6_addr.netmask),
        }
    }
    pub fn broadcast(&self) -> Option<IpAddr> {
        match *self {
            IfAddr::V4(ref ifv4_addr) => ifv4_addr.broadcast.map(IpAddr::V4),
            IfAddr::V6(ref ifv6_addr) => ifv6_addr.broadcast.map(IpAddr::V6),
        }
    }

    #[must_use]
    pub fn network(&self) -> IfAddr {
        match *self {
            IfAddr::V4(ref ifv4_addr) => IfAddr::V4(ifv4_addr.network()),
            IfAddr::V6(ref ifv6_addr) => IfAddr::V6(ifv6_addr.network()),
        }
    }

    #[must_use]
    pub fn contains_address(&self, addr: IpAddr) -> bool {
        match *self {
            IfAddr::V4(ref ifv4_addr) => match addr {
                IpAddr::V4(ipv4_addr) => ifv4_addr.contains_address(ipv4_addr),
                IpAddr::V6(_consensus_count) => false,
            },
            IfAddr::V6(ref ifv6_addr) => match addr {
                IpAddr::V4(_) => false,
                IpAddr::V6(ipv6_addr) => ifv6_addr.contains_address(ipv6_addr),
            },
        }
    }
}

/// Details about the ipv4 address of an interface on this host.
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Clone, Serialize, Deserialize)]
pub struct Ifv4Addr {
    /// The IP address of the interface.
    pub ip: Ipv4Addr,
    /// The netmask of the interface.
    pub netmask: Ipv4Addr,
    /// The broadcast address of the interface.
    pub broadcast: Option<Ipv4Addr>,
}

impl Ifv4Addr {
    /// Convert ip address to the address of the network itself by applying the netmask
    #[must_use]
    pub fn network(&self) -> Ifv4Addr {
        let v4 = self.ip.octets();
        let v4mask = self.netmask.octets();
        let ip = Ipv4Addr::new(
            v4[0] & v4mask[0],
            v4[1] & v4mask[1],
            v4[2] & v4mask[2],
            v4[3] & v4mask[3],
        );
        Ifv4Addr {
            ip,
            netmask: self.netmask,
            broadcast: self.broadcast,
        }
    }

    #[must_use]
    pub fn contains_address(&self, addr: Ipv4Addr) -> bool {
        let v4mask = self.netmask.octets();

        let self_v4 = self.ip.octets();
        let self_ip = Ipv4Addr::new(
            self_v4[0] & v4mask[0],
            self_v4[1] & v4mask[1],
            self_v4[2] & v4mask[2],
            self_v4[3] & v4mask[3],
        );

        let addr_v4 = addr.octets();
        let addr_ip = Ipv4Addr::new(
            addr_v4[0] & v4mask[0],
            addr_v4[1] & v4mask[1],
            addr_v4[2] & v4mask[2],
            addr_v4[3] & v4mask[3],
        );

        self_ip == addr_ip
    }
}

/// Details about the ipv6 address of an interface on this host.
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Clone, Serialize, Deserialize)]
pub struct Ifv6Addr {
    /// The IP address of the interface.
    pub ip: Ipv6Addr,
    /// The netmask of the interface.
    pub netmask: Ipv6Addr,
    /// The broadcast address of the interface.
    pub broadcast: Option<Ipv6Addr>,
}

impl Ifv6Addr {
    /// Convert ip address to the address of the network itself by applying the netmask
    #[must_use]
    pub fn network(&self) -> Ifv6Addr {
        let v6 = self.ip.segments();
        let v6mask = self.netmask.segments();
        let ip = Ipv6Addr::new(
            v6[0] & v6mask[0],
            v6[1] & v6mask[1],
            v6[2] & v6mask[2],
            v6[3] & v6mask[3],
            v6[4] & v6mask[4],
            v6[5] & v6mask[5],
            v6[6] & v6mask[6],
            v6[7] & v6mask[7],
        );
        Ifv6Addr {
            ip,
            netmask: self.netmask,
            broadcast: self.broadcast,
        }
    }

    #[must_use]
    pub fn contains_address(&self, addr: Ipv6Addr) -> bool {
        let v6mask = self.netmask.segments();

        let self_v6 = self.ip.segments();
        let self_ip = Ipv6Addr::new(
            self_v6[0] & v6mask[0],
            self_v6[1] & v6mask[1],
            self_v6[2] & v6mask[2],
            self_v6[3] & v6mask[3],
            self_v6[4] & v6mask[4],
            self_v6[5] & v6mask[5],
            self_v6[6] & v6mask[6],
            self_v6[7] & v6mask[7],
        );

        let addr_v6 = addr.segments();
        let addr_ip = Ipv6Addr::new(
            addr_v6[0] & v6mask[0],
            addr_v6[1] & v6mask[1],
            addr_v6[2] & v6mask[2],
            addr_v6[3] & v6mask[3],
            addr_v6[4] & v6mask[4],
            addr_v6[5] & v6mask[5],
            addr_v6[6] & v6mask[6],
            addr_v6[7] & v6mask[7],
        );

        self_ip == addr_ip
    }
}

/// Some of the flags associated with an interface.
#[derive(
    Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash, Clone, Copy, Serialize, Deserialize,
)]
pub struct InterfaceFlags {
    pub is_loopback: bool,
    pub is_running: bool,
    pub is_point_to_point: bool,
    pub has_default_route: bool,
}

/// Some of the flags associated with an address.
#[derive(
    Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash, Clone, Copy, Serialize, Deserialize,
)]
pub struct AddressFlags {
    // common flags
    pub is_dynamic: bool,
    // ipv6 flags
    pub is_temporary: bool,
    pub is_preferred: bool,
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct InterfaceAddress {
    pub if_addr: IfAddr,
    pub flags: AddressFlags,
}

use core::cmp::Ordering;

// less is less preferable, greater is more preferable
impl Ord for InterfaceAddress {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.if_addr, &other.if_addr) {
            (IfAddr::V4(a), IfAddr::V4(b)) => {
                // global scope addresses are better
                let ret = ipv4addr_is_global(&a.ip).cmp(&ipv4addr_is_global(&b.ip));
                if ret != Ordering::Equal {
                    return ret;
                }
                // local scope addresses are better
                let ret = ipv4addr_is_private(&a.ip).cmp(&ipv4addr_is_private(&b.ip));
                if ret != Ordering::Equal {
                    return ret;
                }
                // non-dynamic addresses are better
                let ret = (!self.flags.is_dynamic).cmp(&!other.flags.is_dynamic);
                if ret != Ordering::Equal {
                    return ret;
                }
            }
            (IfAddr::V6(a), IfAddr::V6(b)) => {
                // preferred addresses are better
                let ret = self.flags.is_preferred.cmp(&other.flags.is_preferred);
                if ret != Ordering::Equal {
                    return ret;
                }
                // non-temporary address are better
                let ret = (!self.flags.is_temporary).cmp(&!other.flags.is_temporary);
                if ret != Ordering::Equal {
                    return ret;
                }
                // global scope addresses are better
                let ret = ipv6addr_is_global(&a.ip).cmp(&ipv6addr_is_global(&b.ip));
                if ret != Ordering::Equal {
                    return ret;
                }
                // unique local unicast addresses are better
                let ret = ipv6addr_is_unique_local(&a.ip).cmp(&ipv6addr_is_unique_local(&b.ip));
                if ret != Ordering::Equal {
                    return ret;
                }
                // unicast site local addresses are better
                let ret = ipv6addr_is_unicast_site_local(&a.ip)
                    .cmp(&ipv6addr_is_unicast_site_local(&b.ip));
                if ret != Ordering::Equal {
                    return ret;
                }
                // unicast link local addresses are better
                let ret = ipv6addr_is_unicast_link_local(&a.ip)
                    .cmp(&ipv6addr_is_unicast_link_local(&b.ip));
                if ret != Ordering::Equal {
                    return ret;
                }
                // non-dynamic addresses are better
                let ret = (!self.flags.is_dynamic).cmp(&!other.flags.is_dynamic);
                if ret != Ordering::Equal {
                    return ret;
                }
            }
            (IfAddr::V4(a), IfAddr::V6(b)) => {
                // If the IPv6 address is preferred and not temporary, compare if it is global scope
                if other.flags.is_preferred && !other.flags.is_temporary {
                    let ret = ipv4addr_is_global(&a.ip).cmp(&ipv6addr_is_global(&b.ip));
                    if ret != Ordering::Equal {
                        return ret;
                    }
                }

                // Default, prefer IPv4 because many IPv6 addresses are not actually routed
                return Ordering::Greater;
            }
            (IfAddr::V6(a), IfAddr::V4(b)) => {
                // If the IPv6 address is preferred and not temporary, compare if it is global scope
                if self.flags.is_preferred && !self.flags.is_temporary {
                    let ret = ipv6addr_is_global(&a.ip).cmp(&ipv4addr_is_global(&b.ip));
                    if ret != Ordering::Equal {
                        return ret;
                    }
                }

                // Default, prefer IPv4 because many IPv6 addresses are not actually routed
                return Ordering::Less;
            }
        }
        // stable sort
        let ret = self.if_addr.cmp(&other.if_addr);
        if ret != Ordering::Equal {
            return ret;
        }
        self.flags.cmp(&other.flags)
    }
}
impl PartialOrd for InterfaceAddress {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl InterfaceAddress {
    #[must_use]
    pub fn new(if_addr: IfAddr, flags: AddressFlags) -> Self {
        Self { if_addr, flags }
    }

    #[must_use]
    pub fn if_addr(&self) -> &IfAddr {
        &self.if_addr
    }

    #[must_use]
    pub fn is_temporary(&self) -> bool {
        self.flags.is_temporary
    }

    #[must_use]
    pub fn is_dynamic(&self) -> bool {
        self.flags.is_dynamic
    }

    #[must_use]
    pub fn is_preferred(&self) -> bool {
        self.flags.is_preferred
    }
}

// #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// enum NetworkInterfaceType {
//     Mobile,     // Least preferable, usually metered and slow
//     Unknown,    // Everything else if we can't detect the type
//     Wireless,   // Wifi is usually free or cheap and medium speed
//     Wired,      // Wired is usually free or cheap and high speed
// }

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub flags: InterfaceFlags,
    pub addrs: Vec<InterfaceAddress>,
}

impl fmt::Debug for NetworkInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NetworkInterface")
            .field("name", &self.name)
            .field("flags", &self.flags)
            .field("addrs", &self.addrs)
            .finish()?;
        if f.alternate() {
            writeln!(f)?;
            writeln!(f, "// primary_ipv4: {:?}", self.primary_ipv4())?;
            writeln!(f, "// primary_ipv6: {:?}", self.primary_ipv6())?;
        }
        Ok(())
    }
}
impl NetworkInterface {
    #[must_use]
    pub fn new(name: String, flags: InterfaceFlags) -> Self {
        Self {
            name,
            flags,
            addrs: Vec::new(),
        }
    }
    #[must_use]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    #[must_use]
    pub fn is_loopback(&self) -> bool {
        self.flags.is_loopback
    }

    #[must_use]
    pub fn is_point_to_point(&self) -> bool {
        self.flags.is_point_to_point
    }

    #[must_use]
    pub fn is_running(&self) -> bool {
        self.flags.is_running
    }

    #[must_use]
    pub fn has_default_route(&self) -> bool {
        self.flags.has_default_route
    }

    #[must_use]
    pub fn primary_ipv4(&self) -> Option<InterfaceAddress> {
        let mut ipv4addrs: Vec<&InterfaceAddress> = self
            .addrs
            .iter()
            .filter(|a| matches!(a.if_addr(), IfAddr::V4(_)))
            .collect();
        ipv4addrs.sort();
        ipv4addrs.last().cloned().cloned()
    }

    #[must_use]
    pub fn primary_ipv6(&self) -> Option<InterfaceAddress> {
        let mut ipv6addrs: Vec<&InterfaceAddress> = self
            .addrs
            .iter()
            .filter(|a| matches!(a.if_addr(), IfAddr::V6(_)))
            .collect();
        ipv6addrs.sort();
        ipv6addrs.last().cloned().cloned()
    }
}

pub struct NetworkInterfacesInner {
    valid: bool,
    interfaces: BTreeMap<String, NetworkInterface>,
    interface_address_cache: Arc<Vec<IfAddr>>,
}

#[derive(Clone)]
pub struct NetworkInterfaces {
    inner: Arc<Mutex<NetworkInterfacesInner>>,
}

impl fmt::Debug for NetworkInterfaces {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.inner.lock();
        f.debug_struct("NetworkInterfaces")
            .field("valid", &inner.valid)
            .field("interfaces", &inner.interfaces)
            .finish()?;
        if f.alternate() {
            writeln!(f)?;
            writeln!(
                f,
                "// stable_addresses: {:?}",
                inner.interface_address_cache
            )?;
        }
        Ok(())
    }
}

impl Default for NetworkInterfaces {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkInterfaces {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(NetworkInterfacesInner {
                valid: false,
                interfaces: BTreeMap::new(),
                interface_address_cache: Arc::new(Vec::new()),
            })),
        }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        let inner = self.inner.lock();
        inner.valid
    }
    pub fn clear(&self) {
        let mut inner = self.inner.lock();

        inner.interfaces.clear();
        inner.interface_address_cache = Arc::new(Vec::new());
        inner.valid = false;
    }
    // returns false if refresh had no changes, true if changes were present
    pub async fn refresh(&self) -> std::io::Result<bool> {
        let mut last_interfaces = {
            let mut last_interfaces = BTreeMap::<String, NetworkInterface>::new();
            let mut platform_support = PlatformSupport::new();
            platform_support
                .get_interfaces(&mut last_interfaces)
                .await?;
            last_interfaces
        };

        let mut inner = self.inner.lock();
        core::mem::swap(&mut inner.interfaces, &mut last_interfaces);
        inner.valid = true;

        if last_interfaces != inner.interfaces {
            // get last address cache
            let old_interface_addresses = inner.interface_address_cache.clone();

            // redo the address cache
            Self::cache_interface_addresses(&mut inner);

            // See if our best addresses have changed
            if old_interface_addresses != inner.interface_address_cache {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub fn with_interfaces<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&BTreeMap<String, NetworkInterface>) -> R,
    {
        let inner = self.inner.lock();
        f(&inner.interfaces)
    }

    #[must_use]
    pub fn interface_addresses(&self) -> Arc<Vec<IfAddr>> {
        let inner = self.inner.lock();
        inner.interface_address_cache.clone()
    }

    /////////////////////////////////////////////

    fn cache_interface_addresses(inner: &mut NetworkInterfacesInner) {
        // Reduce interfaces to their best routable ip addresses
        let mut intf_addrs = Vec::new();
        for intf in inner.interfaces.values() {
            if !intf.is_running() ||
            // || !intf.has_default_route()
            intf.is_loopback()
            // || intf.is_point_to_point() // xxx: iOS cellular is 'point-to-point'
            {
                continue;
            }

            for addr in &intf.addrs {
                if addr.is_temporary() {
                    continue;
                }

                intf_addrs.push(addr);
            }

            // if let Some(pipv4) = intf.primary_ipv4() {
            //     // Skip temporary addresses because they're going to change
            //     if !pipv4.is_temporary() {
            //         intf_addrs.push(pipv4);
            //     }
            // }
            // if let Some(pipv6) = intf.primary_ipv6() {
            //     // Skip temporary addresses because they're going to change
            //     if !pipv6.is_temporary() {
            //         intf_addrs.push(pipv6);
            //     }
            // }
        }

        // Sort one more time to get the best interface addresses overall
        let mut interface_addresses = intf_addrs
            .iter()
            .map(|x| x.if_addr().clone())
            .collect::<Vec<_>>();

        interface_addresses.sort();
        interface_addresses.dedup();

        // Now export just the addresses
        inner.interface_address_cache = Arc::new(interface_addresses);
    }
}
