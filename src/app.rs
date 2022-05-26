use clap::{AppSettings, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(name = env!("CARGO_PKG_NAME"))]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(author = env!("CARGO_PKG_AUTHORS"))]
#[clap(about = "Host EDitor")]
#[clap(
    long_about = "Host EDitor allows you to manipulate the /etc/hosts file. It will manage adding new hosts and removing old entries. Any entry added will be validated (valid ip, non-existing previous entry)."
)]
pub struct Cli {
    /// Instead of /etc/hosts, use this file (testing)
    #[clap(long, required = false, default_value = "/etc/hosts")]
    pub file: String,
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Verify the integrity of the hosts file
    Verify {},
    /// List your current hostfile
    Show {},
    /// Add a host to your hostfile
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Add {
        /// Hostname to add to the hostfile
        #[clap(required = true, index = 1)]
        hostname: String,
        /// IP address of the host
        #[clap(required = false, index = 2)]
        ip: Option<String>,
    },
    /// Replace the IP address for a hostname in your hostfile
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Replace {
        /// Hostname of the entry to replace
        #[clap(required = true, index = 1)]
        hostname: String,
        /// IP address to change to
        #[clap(required = true, index = 2)]
        ip: Option<String>,
    },
    /// Delete a host from your hostfile
    Delete {
        /// IP or hostname to remove
        #[clap(required = true, index = 1)]
        entry: String,
    },
}
