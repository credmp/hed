use faccess::PathExt;
use std::io::Write;
use std::{
    fs::{self, File},
    io::{Error, ErrorKind},
    net::IpAddr,
    path::Path,
};
use termion::color;

use regex::Regex;

use crate::errors::ApplicationError;
use crate::hostentry::HostEntry;
use crate::utils::read_lines;

#[derive(Debug)]
pub struct HostFile {
    pub filename: String,
    pub entries: Option<Vec<HostEntry>>,
}

impl HostFile {
    pub fn add_host_entry(&mut self, entry: HostEntry) {
        if self.entries.is_some() {
            let mut e: Vec<HostEntry> = self.entries.as_ref().unwrap().clone();
            e.push(entry);
            self.entries = Some(e);
        } else {
            self.entries = Some(vec![entry]);
        }
    }

    pub fn write(&self) -> Result<(), ApplicationError> {
        //println!("Writing to file {}", self.filename);

        let path = Path::new(&self.filename);
        if !path.writable() {
            if let Err(e) = sudo::escalate_if_needed() {
                return Err(ApplicationError::HostFileUnwritable(e.to_string()));
            }
        }

        if let Err(e) = fs::copy(&self.filename, format!("{}.bak", self.filename)) {
            return Err(ApplicationError::BackupFileWriteFailed(e.to_string()));
        }

        let display = path.display();

        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };

        for item in self.entries.as_ref().unwrap() {
            if let Err(why) = writeln!(file, "{}", item) {
                panic!("couldn't write to {}: {}", display, why)
            }
        }
        Ok(())
    }

    pub fn parse(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        //println!("Reading file {}", self.filename);
        match read_lines(&self.filename) {
            Ok(lines) => {
                self.entries = lines
                    .map(|l| {
                        match HostFile::parse_line(l.unwrap_or_else(|_e| "".to_string()).as_str()) {
                            Ok(s) => Some(s),
                            _ => None,
                        }
                    })
                    .collect();
                Ok(())
            }
            Err(e) => Err(Box::new(Error::new(ErrorKind::Other, e))),
        }
    }

    fn parse_line(input: &str) -> Result<HostEntry, Box<dyn std::error::Error>> {
        let comment = Regex::new(r"^#\s*(?P<c>.+)\s*$").unwrap();
        let entry =
            Regex::new(r"^(?P<ip>.+?)\s+(?P<name>.+?)(\s+(?P<aliasses>[^#]+))?(#\s*(?P<c>.*))?$")
                .unwrap();
        if comment.is_match(input) {
            Ok(comment
                .captures(input)
                .map(|cap| HostEntry {
                    ip: None,
                    name: None,
                    aliasses: None,
                    comment: cap.name("c").map(|t| String::from(t.as_str().trim())),
                })
                .unwrap())
        } else if entry.is_match(input) {
            let caps = entry.captures(input).unwrap();
            let ip_str = caps.name("ip").map(|t| t.as_str()).unwrap();

            let ip: Option<IpAddr> = match ip_str.parse() {
                Ok(x) => Some(x),
                _ => None,
            };

            let name = caps.name("name").map(|t| String::from(t.as_str().trim()));
            let alias = caps
                .name("aliasses")
                .map(|t| String::from(t.as_str().trim()));
            let alias_vec: Option<Vec<String>> = alias.map(|x| {
                x.split_whitespace()
                    .map(String::from)
                    .collect::<Vec<String>>()
            });
            let comment = caps.name("c").map(|t| String::from(t.as_str().trim()));
            Ok(HostEntry {
                ip,
                name,
                aliasses: alias_vec,
                comment,
            })
        } else {
            Ok(HostEntry {
                ip: None,
                name: None,
                aliasses: None,
                comment: None,
            })
        }
    }

    pub(crate) fn remove_ip(&mut self, entry: Option<&str>) {
        let ip: IpAddr = match entry.unwrap().parse() {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Invalid IP address given: {}", e);
                return;
            }
        };

        if self.entries.is_some() {
            let en = self.entries.as_ref().unwrap().clone();
            self.entries = Some(
                en.into_iter()
                    .filter(|he| {
                        if he.ip.is_some() {
                            he.ip.unwrap() != ip
                        } else {
                            true
                        }
                    })
                    .collect::<Vec<_>>(),
            );
        } else {
            eprintln!("No entries to delete");
        }
    }

    pub(crate) fn remove_name(&mut self, name: Option<&str>) {
        // if the name is the `name` and no aliasses, remove the entry

        if self.entries.is_some() && name.is_some() {
            let en = self.entries.as_ref().unwrap().clone();

            if let Some(n) = name {
                // // filter with 'can delete'.
                // self.entries = Some(
                //     en.into_iter()
                //         .filter(|he| !he.can_delete(n))
                //         .collect::<Vec<_>>(),
                // );

                // if the name is the `name`, find the shortest alias to take its place
                let mut updated: Vec<HostEntry> = vec![];
                for mut entry in en {
                    if !entry.can_delete(n) {
                        updated.push(entry.remove_hostname(n));
                    }
                }

                self.entries = Some(updated);
            }
        } else {
            eprintln!("No entries to delete");
        }

        // else the entry needs to be update, if it is there
    }

    /// Add a new entry to the hosts file
    pub(crate) fn add(
        &mut self,
        hostname: Option<&str>,
        ip: Option<&str>,
    ) -> Result<(), crate::errors::ApplicationError> {
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
            for item in self.entries.iter_mut().flatten() {
                let i = item;

                if i.has_ip(&ip_a) && !i.has_name(hostname.unwrap()) {
                    i.add_alias(hostname.unwrap());
                    return Ok(());
                } else if i.has_ip(&ip_a) {
                    return Err(ApplicationError::IpAlreadyInUse(format!("{}", i)));
                } else if !i.has_ip(&ip_a) && i.has_name(hostname.unwrap()) {
                    return Err(ApplicationError::HostnameAlreadyInUse(format!("{}", i)));
                }
            }
            self.add_host_entry(HostEntry {
                ip: ip_address,
                name: Some(hostname.unwrap().to_string()),
                comment: None,
                aliasses: None,
            });
            Ok(())
        } else {
            for item in self.entries.iter_mut().flatten() {
                let i = item;

                if i.can_resolve_host(hostname.unwrap()) && !i.has_name(hostname.unwrap()) {
                    i.add_alias(hostname.unwrap());
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
    pub(crate) fn replace(
        &mut self,
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

        for item in self.entries.iter_mut().flatten() {
            let i = item;

            if i.name.is_some() && i.name.as_ref().unwrap() == hostname.unwrap() {
                i.ip = ip_address;
                return Ok(());
            }
        }
        Ok(())
    }

    /// Color print the hosts file
    pub(crate) fn show(&self) -> Result<(), ApplicationError> {
        let mut out = std::io::stdout();
        if self.entries.is_some() {
            for item in self.entries.as_ref().unwrap() {
                if let Err(e) = item.color_print(&mut out) {
                    eprintln!("Could not print to stdout.... {}", e);
                }
            }
        }
        Ok(())
    }

    /// Delete an name or IP address from the hostsfile
    pub(crate) fn delete(&mut self, entry: Option<&str>) -> Result<(), ApplicationError> {
        let is_ip = match entry.unwrap().parse::<IpAddr>() {
            Ok(_e) => true,
            Err(_e) => false,
        };

        let c = self.entries.as_ref().unwrap().len();

        if is_ip {
            self.remove_ip(entry);
        } else {
            self.remove_name(entry);
        }

        println!(
            "Removed {}{}{} entries",
            color::Fg(color::Green),
            c - self.entries.as_ref().unwrap().len(),
            color::Fg(color::Reset)
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{hostentry::HostEntry, hostfile::HostFile};
    use std::net::IpAddr;

    #[test]
    fn test_functions() {
        let mut hf = HostFile {
            filename: "/tmp/test".to_string(),
            entries: None,
        };

        assert!(hf.entries.is_none());

        hf.add(Some("arjenwiersma.nl"), Some("127.0.0.1"))
            .expect("Adding host");
        assert!(hf.entries.is_some());

        hf.delete(Some("127.0.0.1")).expect("Should delete");
        assert_eq!(hf.entries.as_ref().unwrap().len(), 0);

        hf.add(Some("arjenwiersma.nl"), Some("127.0.0.1"))
            .expect("Adding host");
        assert!(hf.entries.is_some());

        hf.replace(Some("arjenwiersma.nl"), Some("192.168.0.1"))
            .expect("should replace");
        let e = hf.entries.clone();
        let en = e.unwrap().get(0).unwrap().clone();
        assert_eq!(en.name.unwrap(), "arjenwiersma.nl");
        assert_eq!(
            en.ip.unwrap(),
            "192.168.0.1".parse::<IpAddr>().expect("should read ip")
        );
    }

    #[test]
    fn test_empty_parse_line() {
        if let Ok(he) = HostFile::parse_line("") {
            assert_eq!(he.name, None);
            assert_eq!(he.comment, None);
        } else {
            panic!("Failed to parse valid string");
        }
    }

    #[test]
    fn test_parse_comment() {
        if let Ok(he) = HostFile::parse_line("# testing") {
            assert_eq!(he.name, None);
            assert_eq!(he.comment.unwrap(), "testing");
        } else {
            panic!("Failed to parse valid string");
        }
    }

    #[test]
    fn test_parse_host_entry() {
        if let Ok(he) = HostFile::parse_line("127.0.0.1 localhost") {
            assert_eq!(he.name.unwrap(), "localhost");
            assert_eq!(he.comment, None);
            assert_eq!(he.ip.unwrap(), "127.0.0.1".parse::<IpAddr>().unwrap());
        } else {
            panic!("Failed to parse valid string");
        }
    }

    #[test]
    fn test_parse_host_entry_with_alias() {
        if let Ok(he) = HostFile::parse_line("127.0.0.1 localhost alias1 alias2 ") {
            assert_eq!(he.name.unwrap(), "localhost");
            assert_eq!(he.comment, None);
            assert_eq!(he.ip.unwrap(), "127.0.0.1".parse::<IpAddr>().unwrap());
            assert_eq!(he.aliasses.unwrap(), vec!["alias1", "alias2"]);
        } else {
            panic!("Failed to parse valid string");
        }
    }

    #[test]
    fn test_parse_host_entry_with_alias_and_comment() {
        if let Ok(he) = HostFile::parse_line("127.0.0.1 localhost alias1 alias2 # testing") {
            assert_eq!(he.name.unwrap(), "localhost");
            assert_eq!(he.comment.unwrap(), "testing");
            assert_eq!(he.ip.unwrap(), "127.0.0.1".parse::<IpAddr>().unwrap());
            assert_eq!(he.aliasses.unwrap(), vec!["alias1", "alias2"]);
        } else {
            panic!("Failed to parse valid string");
        }
    }
}
