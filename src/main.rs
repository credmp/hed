use std::{net::IpAddr, process::exit};

use clap::{App, Arg};
pub use color_eyre::eyre::Result;
use errors::ApplicationError;
use termion::color;

use crate::{hostentry::HostEntry, hostfile::HostFile};

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
        .long_about("Host EDitor allows you to maniuplate the /etc/hosts file. It will manage adding new hosts and removing old entries. Any entry added will be validated (valid ip, non-existing previous entry).")
        .setting(clap::AppSettings::ColoredHelp)
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::new("file")
                .long("file")
                .required(false)
                .takes_value(true)
                .about("Instead of /etc/hosts, use this file (testing)"),
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
                        .about("Hostname to add to the hostfile"),
                )
                .arg(
                    Arg::new("ip")
                        .index(2)
                        .required(false)
                        .about("Optional: ip address of the hostname"),
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
                        .about("Hostname of the entry to replace"),
                )
                .arg(
                    Arg::new("ip")
                        .index(2)
                        .required(true)
                        .about("IP address to change to"),
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
                        .about("IP or Hostname to remove"),
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
        Some("show") => show(hf),
        Some("add") => {
            let mymatches = matches
                .subcommand_matches("add")
                .expect("Cannot be a subcommand and not be a subcommand");

            let ip = if mymatches.is_present("ip") {
                mymatches.value_of("ip")
            } else {
                None
            };
            add(hf, mymatches.value_of("hostname"), ip)
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
            replace(hf, mymatches.value_of("hostname"), ip)
        }
        Some("delete") => {
            let mymatches = matches
                .subcommand_matches("delete")
                .expect("Cannot be a subcommand and not be a subcommand");

            delete(hf, mymatches.value_of("entry"))
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

/// Add a new entry to the hosts file
fn add(mut hf: HostFile, hostname: Option<&str>, ip: Option<&str>) -> Result<(), ApplicationError> {
    let ip_address: Option<IpAddr> = match ip {
        Some(x) => match x.parse() {
            Ok(y) => Some(y),
            Err(_e) => return Err(ApplicationError::IpAddressConversion()),
        },
        _ => None,
    };

    if hostname.is_none() {
        return Err(ApplicationError::NoHostnameGiven());
    }

    // if IP address is given, find a matching hostentry to add a alias
    //    no ip? add new entry
    // if only a name is given, find a HostEntry already serving a tld
    //    if none are found, err
    if let Some(ip_a) = ip_address {
        for item in hf.entries.iter_mut().flatten() {
            let i = item;

            if i.has_ip(&ip_a) && !i.has_name(hostname.unwrap()) {
                i.add_alias(hostname.unwrap());
                if let Err(e) = hf.write() {
                    return Err(ApplicationError::HostFileUnwritable(e.to_string()));
                }
                return Ok(());
            } else if i.has_ip(&ip_a) {
                return Err(ApplicationError::IpAlreadyInUse(format!("{}", i)));
            } else if !i.has_ip(&ip_a) && i.has_name(hostname.unwrap()) {
                return Err(ApplicationError::HostnameAlreadyInUse(format!("{}", i)));
            }
        }
        hf.add_host_entry(HostEntry {
            ip: ip_address,
            name: Some(hostname.unwrap().to_string()),
            comment: None,
            aliasses: None,
        });
        if let Err(e) = hf.write() {
            return Err(ApplicationError::HostFileUnwritable(e.to_string()));
        }
        Ok(())
    } else {
        for item in hf.entries.iter_mut().flatten() {
            let i = item;

            if i.can_resolve_host(hostname.unwrap()) && !i.has_name(hostname.unwrap()) {
                i.add_alias(hostname.unwrap());
                if let Err(e) = hf.write() {
                    return Err(ApplicationError::HostFileUnwritable(e.to_string()));
                }
                return Ok(());
            } else if i.has_name(hostname.unwrap()) {
                eprintln!("Hostname already exists in the hostfile");
                // It already exists
                return Ok(());
            }
        }

        Err(ApplicationError::NoParentDomain())
    }
}

/// Replace the IP address for a record, will include all of the aliasses as well
fn replace(
    mut hf: HostFile,
    hostname: Option<&str>,
    ip: Option<&str>,
) -> Result<(), ApplicationError> {
    let ip_address: Option<IpAddr> = match ip {
        Some(x) => match x.parse() {
            Ok(y) => Some(y),
            Err(_e) => return Err(ApplicationError::IpAddressConversion()),
        },
        _ => None,
    };

    if hostname.is_none() {
        return Err(ApplicationError::FileABugReport());
    }

    for item in hf.entries.iter_mut().flatten() {
        let i = item;

        if i.name.is_some() && i.name.as_ref().unwrap() == hostname.unwrap() {
            i.ip = ip_address;
            if let Err(e) = hf.write() {
                return Err(ApplicationError::HostFileUnwritable(e.to_string()));
            }
            return Ok(());
        }
    }
    Ok(())
}

/// Color print the hosts file
fn show(hf: HostFile) -> Result<(), ApplicationError> {
    for item in hf.entries.unwrap_or_else(|| vec![]) {
        item.color_print();
    }

    Ok(())
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

/// Delete an name or IP address from the hostsfile
fn delete(mut hf: HostFile, entry: Option<&str>) -> Result<(), ApplicationError> {
    let is_ip = match entry.unwrap().parse::<IpAddr>() {
        Ok(_e) => true,
        Err(_e) => false,
    };

    let c = hf.entries.as_ref().unwrap().len();

    if is_ip {
        hf.remove_ip(entry);
    } else {
        hf.remove_name(entry);
    }

    if let Err(e) = hf.write() {
        return Err(ApplicationError::HostFileUnwritable(e.to_string()));
    }

    println!(
        "Removed {}{}{} entries",
        color::Fg(color::Green),
        c - hf.entries.as_ref().unwrap().len(),
        color::Fg(color::Reset)
    );

    Ok(())
}

mod exits {

    /// Exit code for when exa runs OK.
    pub const SUCCESS: i32 = 0;

    /// Exit code for when there was at least one I/O error during execution.
    pub const RUNTIME_ERROR: i32 = 1;
}
