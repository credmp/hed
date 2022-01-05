use std::process::exit;

use clap::{App, Arg};
pub(crate) use color_eyre::eyre::Result;
use errors::ApplicationError;
use termion::color;

use crate::hostfile::HostFile;

pub mod errors;
pub mod hostentry;
pub mod hostfile;
pub mod utils;

fn main() {
    if let Err(e) = color_eyre::install() {
        eprintln!("Could not setup error handling: {}", e);
        exit(exits::RUNTIME_ERROR);
    }

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Host EDitor")
        .long_about("Host EDitor allows you to manipulate the /etc/hosts file. It will manage adding new hosts and removing old entries. Any entry added will be validated (valid ip, non-existing previous entry).")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::new("file")
                .long("file")
                .required(false)
                .takes_value(true)
                .help("Instead of /etc/hosts, use this file (testing)"),
        )
        .subcommand(
            App::new("verify")
                .about("Verify the integrity of the hosts file")
                .version("0.1"),
        )
        .subcommand(
            App::new("show")
                .about("List your current hostfile")
                .version("0.1"),
        )
        .subcommand(
            App::new("add")
                .about("Add a host to your hostfile")
                .version("0.1")
                .arg(
                    Arg::new("hostname")
                        .index(1)
                        .required(true)
                        .help("Hostname to add to the hostfile"),
                )
                .arg(
                    Arg::new("ip")
                        .index(2)
                        .required(false)
                        .help("Optional: ip address of the hostname"),
                ),
        )
        .subcommand(
            App::new("replace")
                .about("Replace the IP address for a hostname in your hostfile")
                .version("0.1")
                .arg(
                    Arg::new("hostname")
                        .index(1)
                        .required(true)
                        .help("Hostname of the entry to replace"),
                )
                .arg(
                    Arg::new("ip")
                        .index(2)
                        .required(true)
                        .help("IP address to change to"),
                ),
        )
        .subcommand(
            App::new("delete")
                .about("Delete a host from your hostfile")
                .version("0.1")
                .arg(
                    Arg::new("entry")
                        .index(1)
                        .required(true)
                        .help("IP or Hostname to remove"),
                ),
        )
        .get_matches();

    let filename = get_filename(&matches);

    let mut hf = HostFile {
        filename: filename.to_string(),
        entries: None,
    };

    if let Err(e) = hf.parse() {
        eprintln!("Failed to parse the hostfile, this should not happen unless you are using --file to override the file. The error message is: {}", e);
        exit(exits::RUNTIME_ERROR);
    }

    let res = match matches.subcommand_name() {
        Some("verify") => verify(hf),
        Some("show") => hf.show(),
        Some("add") => {
            let mymatches = matches
                .subcommand_matches("add")
                .expect("Cannot be a subcommand and not be a subcommand");

            let ip = if mymatches.is_present("ip") {
                mymatches.value_of("ip")
            } else {
                None
            };
            match hf.add(mymatches.value_of("hostname"), ip) {
                Ok(()) => hf.write(),
                Err(e) => {
                    eprintln!("Failed to process command: {}", e);
                    exit(exits::RUNTIME_ERROR);
                }
            }
        }
        Some("replace") => {
            let mymatches = matches
                .subcommand_matches("replace")
                .expect("Cannot be a subcommand and not be a subcommand");

            let ip = if mymatches.is_present("ip") {
                mymatches.value_of("ip")
            } else {
                None
            };
            match hf.replace(mymatches.value_of("hostname"), ip) {
                Ok(()) => hf.write(),
                Err(e) => {
                    eprintln!("Failed to process command: {}", e);
                    exit(exits::RUNTIME_ERROR);
                }
            }
        }
        Some("delete") => {
            let mymatches = matches
                .subcommand_matches("delete")
                .expect("Cannot be a subcommand and not be a subcommand");

            match hf.delete(mymatches.value_of("entry")) {
                Ok(()) => hf.write(),
                Err(e) => {
                    eprintln!("Failed to process command: {}", e);
                    exit(exits::RUNTIME_ERROR);
                }
            }
        }
        Some(x) => {
            println!("Unimplemented command {} called", x);
            unimplemented!()
        }
        _ => {
            println!("No subcommand given");
            unimplemented!()
        }
    };

    match res {
        Ok(_) => {
            exit(exits::SUCCESS);
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(exits::RUNTIME_ERROR);
        }
    };
}

fn get_filename(matches: &clap::ArgMatches) -> &str {
    match matches.value_of("file") {
        Some(x) => x,
        _ => "/etc/hosts",
    }
}

/// Verify that the host file is parsable
fn verify(hf: HostFile) -> Result<(), ApplicationError> {
    println!(
        "Hostsfile is readable and contains {}{}{} entries.",
        color::Fg(color::Green),
        hf.entries.unwrap_or_else(Vec::new).len(),
        color::Fg(color::Reset),
    );
    Ok(())
}

mod exits {

    /// Exit code for when exa runs OK.
    pub const SUCCESS: i32 = 0;

    /// Exit code for when there was at least one I/O error during execution.
    pub const RUNTIME_ERROR: i32 = 1;
}
