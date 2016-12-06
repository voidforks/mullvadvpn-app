use net::RemoteAddr;

use std::ffi::{OsString, OsStr};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Child, Stdio};

/// An OpenVPN process builder, providing control over the different arguments that the OpenVPN
/// binary accepts.
pub struct OpenVpnBuilder {
    openvpn_bin: OsString,
    config: Option<PathBuf>,
    remotes: Vec<RemoteAddr>,
}

impl OpenVpnBuilder {
    /// Constructs a new `OpenVpnBuilder` for launching OpenVPN processes from the binary at
    /// `openvpn_bin`.
    pub fn new<P: AsRef<OsStr>>(openvpn_bin: P) -> Self {
        OpenVpnBuilder {
            openvpn_bin: OsString::from(openvpn_bin.as_ref()),
            config: None,
            remotes: vec![],
        }
    }

    /// Sets what configuration file will be given to OpenVPN
    pub fn config<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.config = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the addresses that OpenVPN will connect to. See OpenVPN documentation for how multiple
    /// remotes are handled.
    pub fn remotes(&mut self, remotes: Vec<RemoteAddr>) -> &mut Self {
        self.remotes = remotes;
        self
    }

    /// Executes the OpenVPN process as a child process, returning a handle to it.
    pub fn spawn(&mut self) -> io::Result<Child> {
        let mut command = self.create_command();
        command.args(&self.get_arguments());
        command.spawn()
    }

    fn create_command(&mut self) -> Command {
        let mut command = Command::new(&self.openvpn_bin);
        command.env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command
    }

    /// Returns all arguments that the subprocess would be spawned with.
    pub fn get_arguments(&self) -> Vec<OsString> {
        let mut args = vec![];
        if let Some(ref config) = self.config {
            args.push(OsString::from("--config"));
            args.push(OsString::from(config.as_os_str()));
        }
        for remote in &self.remotes {
            args.push(OsString::from("--remote"));
            args.push(OsString::from(remote.address()));
            args.push(OsString::from(remote.port().to_string()));
        }
        args
    }
}

#[cfg(test)]
mod tests {
    use net::RemoteAddr;
    use std::ffi::OsString;
    use super::OpenVpnBuilder;

    #[test]
    fn no_arguments() {
        let args = OpenVpnBuilder::new("").get_arguments();
        assert_eq!(0, args.len());
    }

    #[test]
    fn passes_one_remote() {
        let remotes = vec![RemoteAddr::new("example.com", 3333)];

        let args = OpenVpnBuilder::new("").remotes(remotes).get_arguments();

        assert!(args.contains(&OsString::from("example.com")));
        assert!(args.contains(&OsString::from("3333")));
    }

    #[test]
    fn passes_two_remotes() {
        let remotes = vec![RemoteAddr::new("127.0.0.1", 998), RemoteAddr::new("fe80::1", 1337)];

        let args = OpenVpnBuilder::new("").remotes(remotes).get_arguments();

        assert!(args.contains(&OsString::from("127.0.0.1")));
        assert!(args.contains(&OsString::from("998")));
        assert!(args.contains(&OsString::from("fe80::1")));
        assert!(args.contains(&OsString::from("1337")));
    }
}