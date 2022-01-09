use faccess::PathExt;
use std::io::Write;
use std::{
    fs::{self, File},
    io::{Error, ErrorKind},
    net::IpAddr,
    path::Path,
};
use termion::color;

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

        self.backup()?;

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

    pub fn backup(&self) -> Result<(), ApplicationError> {
        if let Err(e) = fs::copy(&self.filename, format!("{}.bak", self.filename)) {
            return Err(ApplicationError::BackupFileWriteFailed(e.to_string()));
        }

        Ok(())
    }

    pub fn parse(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        //println!("Reading file {}", self.filename);
        match read_lines(&self.filename) {
            Ok(lines) => {
                self.entries = lines
                    .map(|l| {
                        match l.unwrap_or_else(|_e| "".to_string()).parse() {
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

    pub(crate) fn remove_ip(&mut self, entry: String) {
        let ip: IpAddr = match entry.parse() {
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

    pub(crate) fn remove_name(&mut self, name: String) {
        // if the name is the `name` and no aliasses, remove the entry

        if self.entries.is_some() {
            let en = self.entries.as_ref().unwrap().clone();

            let mut updated: Vec<HostEntry> = vec![];
            for mut entry in en {
                if !entry.can_delete(name.as_str()) {
                    updated.push(entry.remove_hostname(name.as_str()));
                }
            }

                self.entries = Some(updated);
        } else {
            eprintln!("No entries to delete");
        }

        // else the entry needs to be update, if it is there
    }

    /// Add a new entry to the hosts file
    pub(crate) fn add(
        &mut self,
        hostname: String,
        ip: Option<String>,
    ) -> Result<(), crate::errors::ApplicationError> {
        let ip_address: Option<IpAddr> = match ip {
            Some(x) => match x.parse() {
                Ok(y) => Some(y),
                Err(_e) => return Err(ApplicationError::IpAddressConversion()),
            },
            _ => None,
        };

        // if IP address is given, find a matching hostentry to add a alias
        //    no ip? add new entry
        // if only a name is given, find a HostEntry already serving a tld
        //    if none are found, err
        if let Some(ip_a) = ip_address {
            for item in self.entries.iter_mut().flatten() {
                let i = item;

                if i.has_ip(&ip_a) && !i.has_name(hostname.as_str()) {
                    if i.can_hostname_resolve_domain(hostname.as_str()) {
                        i.switch_name_with_alias(hostname.as_str());
                    } else {
                        i.add_alias(hostname.as_str());
                    }
                    return Ok(());
                } else if i.has_ip(&ip_a) {
                    return Err(ApplicationError::IpAlreadyInUse(format!("{}", i)));
                } else if !i.has_ip(&ip_a) && i.has_name(hostname.as_str()) {
                    return Err(ApplicationError::HostnameAlreadyInUse(format!("{}", i)));
                }
            }
            self.add_host_entry(HostEntry {
                ip: ip_address,
                name: Some(hostname),
                comment: None,
                aliasses: None,
            });
            Ok(())
        } else {
            for item in self.entries.iter_mut().flatten() {
                let i = item;

                if i.can_resolve_host(hostname.as_str()) && !i.has_name(hostname.as_str()) {
                    i.add_alias(hostname.as_str());
                    return Ok(());
                } else if i.can_hostname_resolve_domain(hostname.as_str()) {
                    i.switch_name_with_alias(hostname.as_str());
                    return Ok(());
                } else if i.has_name(hostname.as_str()) {
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
        hostname: String,
        ip: Option<String>,
    ) -> Result<(), ApplicationError> {
        let ip_address: Option<IpAddr> = match ip {
            Some(x) => match x.parse() {
                Ok(y) => Some(y),
                Err(_e) => return Err(ApplicationError::IpAddressConversion()),
            },
            _ => None,
        };

        for item in self.entries.iter_mut().flatten() {
            let i = item;

            if i.name.is_some() && i.name.as_ref().unwrap() == hostname.as_str() {
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
    pub(crate) fn delete(&mut self, entry: String) -> Result<(), ApplicationError> {
        let is_ip = match entry.parse::<IpAddr>() {
            Ok(_e) => true,
            Err(_e) => false,
        };

        let c = self.entries.as_ref().unwrap().len();

        if is_ip {
            self.remove_ip(entry);
        } else {
            self.remove_name(entry);
        }

        let updated = c - self.entries.as_ref().unwrap().len();
        if updated > 0 {
            println!(
                "Removed {}{}{} entries",
                color::Fg(color::Green),
                c - self.entries.as_ref().unwrap().len(),
                color::Fg(color::Reset)
            );
        } else {
            println!("No entries were removed, some could have been updated though",);
        }

        Ok(())
    }

    pub(crate) fn import(&mut self, filename: String) -> Result<(), ApplicationError> {
        if std::path::Path::new(filename.as_str()).exists() {
            let entries = match import_from_file(filename) {
                Ok(value) => value,
                Err(value) => return value,
            };

            for he in entries {
                self.add_host_entry(he.unwrap());
            }

            Ok(())
        } else if filename.starts_with("http") {
            let entries = match import_from_url(filename) {
                Ok(value) => value,
                Err(value) => return value,
            };

            for he in entries {
                self.add_host_entry(he.unwrap());
            }
            
            Ok(())
        }  else {
            // else indicate we do not know
            Err(ApplicationError::ImportFilenameUnreadable(filename, "no suitable type".to_string()))
        }
    }
}

fn import_from_url(filename: String) -> Result<Vec<Option<HostEntry>>, Result<(), ApplicationError>> {
    println!("Attempting to retrieve {}", filename);
    let res = match reqwest::blocking::get(filename.as_str()) {
        Ok(body) => {
            match body.text() {
                Ok(t) => Some(t),
                Err(_) => None,
            }
        }
        Err(_) => None,
    };
    println!("Retrieved the remote file, processing {} bytes (can take a bit with larger files due to all the validation)", res.as_ref().unwrap().as_str().len());
    let entries: Vec<Option<HostEntry>> = match res {
        Some(t) => {
            let lines = t.split("\n");
            lines.map(|l| {
                match l.parse() {
                    Ok(s) => Some(s),
                    _ => None,                            
                }
            }).collect()
        }
        None => {
            return Err(Err(ApplicationError::ImportURLInaccessible(filename)));
        }
    };
    Ok(entries)
}

fn import_from_file(filename: String) -> Result<Vec<Option<HostEntry>>, Result<(), ApplicationError>> {
    let entries:Vec<Option<HostEntry>> = match read_lines(filename.as_str()) {
        Ok(lines) => {
            lines
                .map(|l| {
                    match l.unwrap_or_else(|_e| "".to_string()).parse() {
                        Ok(s) => Some(s),
                        _ => None,                            
                    }
                })
                .collect()
        }
        Err(e) => {
            return Err(Err(ApplicationError::ImportFilenameUnreadable(filename, e.to_string())))
        }
    };
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use crate::hostfile::HostFile;
    use std::net::IpAddr;

    #[test]
    fn test_functions() {
        let mut hf = HostFile {
            filename: "/tmp/test".to_string(),
            entries: None,
        };

        assert!(hf.entries.is_none());

        // ADD functions

        // add it
        hf.add(String::from("arjenwiersma.nl"), Some(String::from("127.0.0.1")))
            .expect("Adding host");
        assert!(hf.entries.is_some());

        // remove it by ip
        hf.delete(String::from("127.0.0.1")).expect("Should delete");
        assert_eq!(hf.entries.as_ref().unwrap().len(), 0);

        hf.add(String::from("arjenwiersma.nl"), Some(String::from("127.0.0.1")))
            .expect("Adding host");
        assert!(hf.entries.is_some());

        // remove it by name
        hf.delete(String::from("arjenwiersma.nl")).expect("Should delete");
        assert_eq!(hf.entries.as_ref().unwrap().len(), 0);

        // add a subdomain first
        hf.add(String::from("me.arjenwiersma.nl"), Some(String::from("127.0.0.1")))
            .expect("Adding host");

        // add parent domain after
        hf.add(String::from("arjenwiersma.nl"), Some(String::from("127.0.0.1")))
            .expect("Adding parent domain");

        // ensure the parent domain is the `name` property
        assert!(hf.entries.is_some());
        assert_eq!(hf.entries.as_ref().unwrap().len(), 1);
        let he = hf.entries.as_ref().unwrap().get(0).unwrap();
        assert_eq!(he.name.as_ref().unwrap(), "arjenwiersma.nl");

        // and subdomain the alias
        assert_eq!(
            he.aliasses.as_ref().unwrap().get(0).unwrap(),
            "me.arjenwiersma.nl"
        );

        hf.delete(String::from("127.0.0.1")).expect("Should delete");
        assert_eq!(hf.entries.as_ref().unwrap().len(), 0);

        // parent first
        hf.add(String::from("arjenwiersma.nl"), Some(String::from("127.0.0.1")))
            .expect("Adding host");
        // subdomain second
        hf.add(String::from("demo.arjenwiersma.nl"), Some(String::from("127.0.0.1")))
            .expect("Adding host");
        assert!(hf.entries.is_some());
        assert_eq!(hf.entries.as_ref().unwrap().len(), 1);
        let he = hf.entries.as_ref().unwrap().get(0).unwrap();
        assert_eq!(he.name.as_ref().unwrap(), "arjenwiersma.nl");

        // and subdomain the alias
        assert_eq!(
            he.aliasses.as_ref().unwrap().get(0).unwrap(),
            "demo.arjenwiersma.nl"
        );

        // a second alias is added
        hf.add(String::from("demo2.arjenwiersma.nl"), Some(String::from("127.1.0.1")))
            .expect("Adding host");
        assert_eq!(hf.entries.as_ref().unwrap().len(), 2);

        // REPLACE function

        // replace the ip address of the domain
        hf.replace(String::from("arjenwiersma.nl"), Some(String::from("192.168.0.1")))
            .expect("should replace");
        let e = hf.entries.clone();
        let en = e.unwrap().get(0).unwrap().clone();
        assert_eq!(en.name.unwrap(), "arjenwiersma.nl");
        assert_eq!(
            en.ip.unwrap(),
            "192.168.0.1".parse::<IpAddr>().expect("should read ip")
        );
    }

}
