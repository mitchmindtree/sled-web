use hyper::{self, Server};
use hyper::rt::Future;
use hyper::service::service_fn;
use response::response;
use sled;
use std::net::SocketAddr;
use std::sync::Arc;

// Request strings.

/// Configuration for the server.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The socket address to which the server will bind and listen for HTTP requests.
    ///
    /// Defaults to localhost:3000. E.g. `([127, 0, 0, 1], 3000)`.
    pub addr: SocketAddr,
}

/// A type used for building a `Config`.
#[derive(Clone, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ConfigBuilder {
    pub addr: Option<SocketAddr>,
}

/// Begin building the configuration for the server.
pub fn config() -> ConfigBuilder {
    Default::default()
}

// Implementations.

impl Config {
    /// The default IP address used if a socket address is not specified.
    pub const DEFAULT_IP: [u8; 4] = [127, 0, 0, 1];
    /// The default port used if a socket address is not specified.
    pub const DEFAULT_PORT: u16 = 3000;
    /// The default socket address used if one is not specified.
    pub const DEFAULT_ADDR: ([u8; 4], u16) = (Self::DEFAULT_IP, Self::DEFAULT_PORT);
}

impl ConfigBuilder {
    /// The socket address to which the server will bind and listen for HTTP requests.
    ///
    /// Defaults to localhost:3000. E.g. `([127, 0, 0, 1], 3000)`.
    pub fn addr<T>(&mut self, addr: T) -> &mut Self
    where
        T: Into<SocketAddr>,
    {
        self.addr = Some(addr.into());
        self
    }

    /// Build the `Config` type, replacing `None` values with defaults where necessary.
    pub fn build(&mut self) -> Config {
        let addr = self.addr.take().unwrap_or_else(|| Config::DEFAULT_ADDR.into());
        Config { addr }
    }
}

// Pure functions.

/// Build the hyper `Server` with the given configuration and `sled::Tree`.
///
/// Returns a `Future` representing the `Server`'s computation.
pub fn new(config: Config, tree: Arc<sled::Tree>) -> impl Future<Item = (), Error = hyper::Error> {
    Server::bind(&config.addr)
        .serve(move || {
            let tree = tree.clone();
            service_fn(move |req| {
                response(req, tree.clone())
            })
        })
}

/// Build and run a hyper `Server` using the default runtime with the given configuration and
/// `sled::Tree`.
pub fn run(config: Config, tree: Arc<sled::Tree>) {
    let server = new(config, tree)
        .map_err(|e| eprintln!("error occurred: {}", e));
    hyper::rt::run(server);
}
