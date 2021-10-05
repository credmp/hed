use std::{fmt, net::IpAddr};

#[derive(Debug, Clone)]
pub struct HostEntry {
    pub ip: Option<IpAddr>,
    pub name: Option<String>,
    pub aliasses: Option<Vec<String>>,
    pub comment: Option<String>,
}

impl HostEntry {
    pub fn empty() -> HostEntry {
        HostEntry {
            ip: None,
            name: None,
            aliasses: None,
            comment: None,
        }
    }

    /// Checks if the `name` of `HostEntry` can result the passed `hostname`.
    ///
    /// If `name` is `host.tld` and `hostname` is a subdomain, return true.
    /// If `name` is a subdomain `sub.host.tld` of `hostname`, return false.
    ///
    /// A subdomain is more specific, to rule out overlap, do not change it.
    /// TODO: allow reassigning of `name`
    pub fn can_resolve_host(&self, hostname: &str) -> bool {
        if self.name.is_some() {
            hostname.ends_with(self.name.as_ref().unwrap().as_str())
        } else {
            false
        }
    }

    pub fn has_ip(&self, ip: &IpAddr) -> bool {
        if self.ip.is_some() {
            self.ip.as_ref().unwrap() == ip
        } else {
            false
        }
    }

    pub fn has_name(&self, hostname: &str) -> bool {
        if let Some(x) = &self.name {
            if x.eq(hostname) {
                return true;
            }
        }
        match &self.aliasses {
            Some(y) => {
                for z in y {
                    if z == hostname {
                        return true;
                    }
                }
            }
            _ => {
                return false;
            }
        }
        false
    }

    pub(crate) fn add_alias(&mut self, hostname: &str) {
        if self.aliasses.is_some() {
            let mut alias: Vec<String> = self.aliasses.as_ref().unwrap().clone();
            alias.push(hostname.to_string());
            self.aliasses = Some(alias);
        } else {
            self.aliasses = Some(vec![hostname.to_string()]);
        }
    }
}

impl PartialEq for HostEntry {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip
            && self.name == other.name
            && self.aliasses == other.aliasses
            && self.comment == other.comment
    }
}

// impl PartialOrd for HostEntry {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         todo!()
//     }
// }

impl fmt::Display for HostEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.ip == None && self.comment != None {
            write!(f, "# {}", self.comment.as_ref().unwrap())
        } else if self.ip != None {
            write!(
                f,
                "{}\t{}\t{}",
                self.ip.unwrap(),
                self.name.as_ref().unwrap(),
                self.aliasses.as_ref().unwrap_or(&vec![]).join(" ")
            )
        } else {
            write!(f, "")
        }
    }
}
