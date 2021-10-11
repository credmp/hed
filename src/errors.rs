use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    /// Represents a failure to read the hosts file
    #[error("hostfile is not readable. Reason: {0}")]
    HostFileUnreadable(String),

    #[error("Failed to write the hostfile back to the file. Reason: {0}")]
    HostFileUnwritable(String),

    #[error("Failed to convert the IP address, this is normally due to a typo of perhaps you gave a hostname instead?")]
    IpAddressConversion(),

    #[error("No hostname was given to be added to the hosts file. You should not see this message, if you do, please log an bug report at https://github.com/credmp/hed")]
    NoHostnameGiven(),

    #[error("An ip address with this name already exists:\n{0}")]
    IpAlreadyInUse(String),

    #[error("An entry exists with the hostname, but with a different IP:\n{0}")]
    HostnameAlreadyInUse(String),

    #[error("Could not add host, no parent domain to resolve it. This means that no parent domain exists for the given hostname, try adding it with an IP address, it will be the first entry for this host.\n\nFor instance, if an entry exists called demo.example.com I can not add example.com, as the subdomain is not a parent domain, the other way around will work.")]
    NoParentDomain(),

    #[error("You should not see this message, if you do, please log an bug report at https://github.com/credmp/hed, it is very appreciated!")]
    FileABugReport(),

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
