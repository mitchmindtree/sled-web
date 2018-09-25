use hyper::{self, Body, StatusCode, Uri};
use hyper::client::HttpConnector;
use hyper::rt::{Future, Stream};
use request;
use serde_json;
use std::error::Error as StdError;
use std::fmt;

/// A hyper `Client` wrapper that simplifies communication with the sled `Tree` server.
#[derive( Clone, Debug)]
pub struct Client {
    uri: Uri,
    client: hyper::Client<HttpConnector, Body>,
}

/// The possible errors that may be produced by the `Client` request methods.
#[derive(Debug)]
pub enum Error {
    Hyper(hyper::Error),
    SerdeJson(serde_json::Error),
    Server(String),
}

impl Client {
    /// Create a new `Client` pointing towards the given `Uri`.
    ///
    /// The `Uri` should contain the `Scheme` and `Authority` parts of the URI but not the
    /// following path. This following path will be created as necessary within each of the request
    /// calls.
    pub fn new(uri: Uri) -> Self {
        let client = hyper::Client::new();
        Client { uri, client }
    }

    /// A method for performing the `Get` request.
    ///
    /// Given the key for an entry in the `sled::Tree`, produce a `Future` with the value.
    pub fn get(&self, key: Vec<u8>) -> impl Future<Item = Option<Vec<u8>>, Error = Error> {
        let request = request::get(self.uri.clone(), key);
        self.client
            .request(request)
            .map_err(Error::Hyper)
            .and_then(|response| {
                let status = response.status();
                response
                    .into_body()
                    .concat2()
                    .map_err(Error::Hyper)
                    .and_then(move |chunk| {
                        if status == StatusCode::INTERNAL_SERVER_ERROR {
                            let s = serde_json::from_slice(&chunk).map_err(Error::SerdeJson)?;
                            return Err(Error::Server(s));
                        }
                        serde_json::from_slice(&chunk).map_err(Error::SerdeJson)
                    })
            })
    }

    /// A method for performing the `Del` request.
    ///
    /// Given the key for an entry in the `sled::Tree`, delete the entry and return a `Future` with
    /// the removed value.
    pub fn del(&self, key: Vec<u8>) -> impl Future<Item = Option<Vec<u8>>, Error = Error> {
        let request = request::del(self.uri.clone(), key);
        self.client
            .request(request)
            .map_err(Error::Hyper)
            .and_then(|response| {
                let status = response.status();
                response
                    .into_body()
                    .concat2()
                    .map_err(Error::Hyper)
                    .and_then(move |chunk| {
                        if status == StatusCode::INTERNAL_SERVER_ERROR {
                            let s = serde_json::from_slice(&chunk).map_err(Error::SerdeJson)?;
                            return Err(Error::Server(s));
                        }
                        serde_json::from_slice(&chunk).map_err(Error::SerdeJson)
                    })
            })
    }

    /// A method for performing the `Set` request.
    ///
    /// Send the given key and value to the database for insertion into the `sled::Tree`.
    pub fn set(&self, key: Vec<u8>, value: Vec<u8>) -> impl Future<Item = (), Error = Error> {
        let request = request::set(self.uri.clone(), key, value);
        self.client
            .request(request)
            .map_err(Error::Hyper)
            .and_then(|response| {
                let status = response.status();
                response
                    .into_body()
                    .concat2()
                    .map_err(Error::Hyper)
                    .and_then(move |chunk| {
                        if status == StatusCode::INTERNAL_SERVER_ERROR {
                            let s = serde_json::from_slice(&chunk).map_err(Error::SerdeJson)?;
                            return Err(Error::Server(s));
                        }
                        Ok(())
                    })
            })
    }

    /// A method for performing the `Iter` request.
    ///
    /// The result is a `Stream` of ordered key value pairs.
    pub fn iter(&self) -> impl Stream<Item = (Vec<u8>, Vec<u8>), Error = Error> {
        let request = request::iter(self.uri.clone());
        self.client
            .request(request)
            .map_err(Error::Hyper)
            .map(|response| {
                response
                    .into_body()
                    .map_err(Error::Hyper)
                    .and_then(|chunk| serde_json::from_slice(&chunk).map_err(Error::SerdeJson))
            })
            .flatten_stream()
    }

    /// A method for performing the `Scan` request.
    ///
    /// The result is a `Stream` of ordered key value pairs, starting from the given key.
    pub fn scan(&self, key: Vec<u8>) -> impl Stream<Item = (Vec<u8>, Vec<u8>), Error = Error> {
        let request = request::scan(self.uri.clone(), key);
        self.client
            .request(request)
            .map_err(Error::Hyper)
            .map(|response| {
                response
                    .into_body()
                    .map_err(Error::Hyper)
                    .and_then(|chunk| serde_json::from_slice(&chunk).map_err(Error::SerdeJson))
            })
            .flatten_stream()
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Hyper(ref err) => err.description(),
            Error::SerdeJson(ref err) => err.description(),
            Error::Server(ref s) => s,
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Hyper(ref err) => Some(err),
            Error::SerdeJson(ref err) => Some(err),
            Error::Server(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::Hyper(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeJson(e)
    }
}
