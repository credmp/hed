use std::net::IpAddr;

use clap::{App, Arg};
use color_eyre::eyre::eyre;
pub use color_eyre::eyre::Result;

use crate::{hostentry::HostEntry, hostfile::HostFile};

pub mod hostentry;
pub mod hostfile;
pub mod utils;

fn main() -> Result<()> {
    color_eyre::install()?;

    let matches = App::new("hed")
        .version("0.0.1.2-alpha")
        .author("Arjen Wiersma <arjen@wiersma.org")
        .about("Host EDitor")
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
        .get_matches();

    let res = match matches.subcommand_name() {
        Some("verify") => verify(get_filename(&matches)),
        Some("show") => show(get_filename(&matches)),
        Some("add") => {
            let mymatches = matches
                .subcommand_matches("add")
                .expect("Cannot be a subcommand and not be a subcommand");

            let ip = if mymatches.is_present("ip") {
                mymatches.value_of("ip")
            } else {
                None
            };
            add(get_filename(&matches), mymatches.value_of("hostname"), ip)
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
    res
}

fn get_filename(matches: &clap::ArgMatches) -> &str {
    match matches.value_of("file") {
        Some(x) => x,
        _ => "/etc/hosts",
    }
}

// fn get_hostentries(filename: &str) -> Result<Vec<Option<HostEntry>>> {
//     match parse_file(filename) {
//         Ok(x) => Ok(x),
//         Err(x) => Err(eyre!("{}", x)),
//     }
// }

fn add(filename: &str, hostname: Option<&str>, ip: Option<&str>) -> Result<(), color_eyre::Report> {
    let ip_address: Option<IpAddr> = match ip {
        Some(x) => match x.parse() {
            Ok(y) => Some(y),
            Err(e) => return Err(eyre!(e)),
        },
        _ => None,
    };

    if hostname.is_none() {
        return Err(eyre!("No hostname given"));
    }

    let mut hf = HostFile {
        filename: filename.to_string(),
        entries: None,
    };

    if let Err(x) = hf.parse() {
        return Err(eyre!("Failed to parse file {}", x));
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
                if let Err(x) = hf.write() {
                    return Err(eyre!("Error: {}", x));
                }
                return Ok(());
            }
        }
        hf.add_host_entry(HostEntry {
            ip: ip_address,
            name: Some(hostname.unwrap().to_string()),
            comment: None,
            aliasses: None,
        });
        if let Err(x) = hf.write() {
            return Err(eyre!("Error: {}", x));
        }
        Ok(())
    } else {
        for item in hf.entries.iter_mut().flatten() {
            let i = item;

            if i.can_resolve_host(hostname.unwrap()) && !i.has_name(hostname.unwrap()) {
                i.add_alias(hostname.unwrap());
                if let Err(x) = hf.write() {
                    return Err(eyre!("Error: {}", x));
                }
                return Ok(());
            }
        }

        Err(eyre!("Could not add host, no parent domain to resolve it."))
    }
}

fn show(filename: &str) -> Result<(), color_eyre::Report> {
    let mut hf = HostFile {
        filename: filename.to_string(),
        entries: None,
    };

    match hf.parse() {
        Ok(()) => {
            for item in hf.entries.unwrap_or_else(|| vec![]) {
                println!("{}", item); //item.unwrap_or_else(HostEntry::empty))
            }

            return Ok(());
        }
        Err(x) => Err(eyre!("Failed to parse file {}", x)),
    }
}

fn verify(filename: &str) -> Result<(), color_eyre::Report> {
    let mut hf = HostFile {
        filename: filename.to_string(),
        entries: None,
    };

    match hf.parse() {
        Ok(()) => {
            println!("Hostsfile is readable and contains  entries");
            Ok(())
        }
        Err(x) => Err(eyre!("Failed to parse file {}", x)),
    }
}

#[cfg(test)]
mod test {

    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use crate::hostentry::HostEntry;

    use super::*;

    #[test]
    fn test_ip_addr() {
        let localhost_v4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let localhost_v6 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));

        assert_eq!("127.0.0.1".parse(), Ok(localhost_v4));
        assert_eq!("::1".parse(), Ok(localhost_v6));
    }

    #[test]
    fn test_hostentry_has_name() {
        let he = HostEntry {
            ip: None,
            name: Some(String::from("arjen.wiersma.nl")),
            aliasses: None,
            comment: None,
        };

        assert!(he.has_name("arjen.wiersma.nl"));
        assert!(!he.has_name("arjen2.wiersma.nl"));
        assert!(!he.has_name("wiersma.nl"));

        let ahe = HostEntry {
            ip: None,
            name: Some(String::from("jelle.wiersma.nl")),
            aliasses: Some(vec![
                String::from("arjen.wiersma.nl"),
                String::from("rebecca.wiersma.nl"),
            ]),
            comment: None,
        };
        assert!(ahe.has_name("arjen.wiersma.nl"));
        assert!(ahe.has_name("rebecca.wiersma.nl"));
        assert!(!ahe.has_name("arjen2.wiersma.nl"));
        assert!(!ahe.has_name("wiersma.nl"));
        assert!(ahe.has_name("jelle.wiersma.nl"));
    }
}
