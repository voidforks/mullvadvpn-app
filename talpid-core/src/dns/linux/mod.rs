mod network_manager;
mod resolvconf;
mod static_resolv_conf;
mod systemd_resolved;

use self::{
    network_manager::NetworkManager, resolvconf::Resolvconf, static_resolv_conf::StaticResolvConf,
    systemd_resolved::SystemdResolved,
};
use std::{env, fmt, net::IpAddr, path::Path};


const RESOLV_CONF_PATH: &str = "/etc/resolv.conf";

error_chain! {
    errors {
        NoDnsMonitor {
            description("No suitable DNS monitor implementation detected")
        }
    }

    links {
        Resolvconf(resolvconf::Error, resolvconf::ErrorKind);
        StaticResolvConf(static_resolv_conf::Error, static_resolv_conf::ErrorKind);
        SystemdResolved(systemd_resolved::Error, systemd_resolved::ErrorKind);
        NetworkManager(network_manager::Error, network_manager::ErrorKind);
    }
}

pub struct DnsMonitor {
    inner: Option<DnsMonitorHolder>,
}

impl super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new(_cache_dir: impl AsRef<Path>) -> Result<Self> {
        Ok(DnsMonitor { inner: None })
    }

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        self.reset()?;
        // Creating a new DNS monitor for each set, in case the system changed how it manages DNS.
        let mut inner = DnsMonitorHolder::new()?;
        inner.set(interface, servers)?;
        self.inner = Some(inner);
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        if let Some(mut inner) = self.inner.take() {
            inner.reset()?;
        }
        Ok(())
    }
}

pub enum DnsMonitorHolder {
    Resolvconf(Resolvconf),
    StaticResolvConf(StaticResolvConf),
    SystemdResolved(SystemdResolved),
    NetworkManager(NetworkManager),
}

impl fmt::Display for DnsMonitorHolder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::DnsMonitorHolder::*;
        let name = match self {
            Resolvconf(..) => "resolvconf",
            StaticResolvConf(..) => "/etc/resolv.conf",
            SystemdResolved(..) => "systemd-resolved",
            NetworkManager(..) => "network manager",
        };
        f.write_str(name)
    }
}

impl DnsMonitorHolder {
    fn new() -> Result<Self> {
        let dns_module = env::var_os("TALPID_DNS_MODULE");

        let manager = match dns_module.as_ref().and_then(|value| value.to_str()) {
            Some("static-file") => DnsMonitorHolder::StaticResolvConf(StaticResolvConf::new()?),
            Some("resolvconf") => DnsMonitorHolder::Resolvconf(Resolvconf::new()?),
            Some("systemd") => DnsMonitorHolder::SystemdResolved(SystemdResolved::new()?),
            Some("network-manager") => DnsMonitorHolder::NetworkManager(NetworkManager::new()?),
            Some(_) | None => Self::with_detected_dns_manager()?,
        };
        log::debug!("Managing DNS via {}", manager);
        Ok(manager)
    }

    fn with_detected_dns_manager() -> Result<Self> {
        SystemdResolved::new()
            .map(DnsMonitorHolder::SystemdResolved)
            .or_else(|_| NetworkManager::new().map(DnsMonitorHolder::NetworkManager))
            .or_else(|_| Resolvconf::new().map(DnsMonitorHolder::Resolvconf))
            .or_else(|_| StaticResolvConf::new().map(DnsMonitorHolder::StaticResolvConf))
            .chain_err(|| ErrorKind::NoDnsMonitor)
    }

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        use self::DnsMonitorHolder::*;
        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.set_dns(interface, servers)?,
            StaticResolvConf(ref mut static_resolv_conf) => {
                static_resolv_conf.set_dns(servers.to_vec())?
            }
            SystemdResolved(ref mut systemd_resolved) => {
                systemd_resolved.set_dns(interface, &servers)?
            }
            NetworkManager(ref mut network_manager) => network_manager.set_dns(servers)?,
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        use self::DnsMonitorHolder::*;
        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.reset()?,
            StaticResolvConf(ref mut static_resolv_conf) => static_resolv_conf.reset()?,
            SystemdResolved(ref mut systemd_resolved) => systemd_resolved.reset()?,
            NetworkManager(ref mut network_manager) => network_manager.reset()?,
        }
        Ok(())
    }
}
