use hyper::{self, Body, Request, Response, StatusCode, Uri};
use hyper::client::HttpConnector;
use hyper::rt::{Future, Stream};
use request;
use serde::Deserialize;
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

pub type Key = Vec<u8>;
pub type Value = Vec<u8>;
pub type Entry = (Vec<u8>, Vec<u8>);

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
    pub fn get(&self, key: Key) -> impl Future<Item = Option<Value>, Error = Error> {
        let request = request::get(self.uri.clone(), key);
        request_concat_and_deserialize(self, request)
    }

    /// A method for performing the `Del` request.
    ///
    /// Given the key for an entry in the `sled::Tree`, delete the entry and return a `Future` with
    /// the removed value.
    pub fn del(&self, key: Key) -> impl Future<Item = Option<Value>, Error = Error> {
        let request = request::del(self.uri.clone(), key);
        request_concat_and_deserialize(self, request)
    }

    /// A method for performing the `Set` request.
    ///
    /// Send the given key and value to the database for insertion into the `sled::Tree`.
    pub fn set(&self, key: Key, value: Value) -> impl Future<Item = (), Error = Error> {
        let request = request::set(self.uri.clone(), key, value);
        request_concat_and_deserialize(self, request)
    }

    /// A method for performing the `Iter` request.
    ///
    /// The result is a `Stream` of ordered key value pairs.
    pub fn iter(&self) -> impl Stream<Item = Entry, Error = Error> {
        let request = request::iter(self.uri.clone());
        request_stream_and_deserialize(self, request)
    }

    /// A method for performing the `Scan` request.
    ///
    /// The result is a `Stream` of ordered key value pairs, starting from the given key.
    pub fn scan(&self, key: Key) -> impl Stream<Item = Entry, Error = Error> {
        let request = request::scan(self.uri.clone(), key);
        request_stream_and_deserialize(self, request)
    }

    /// A method for performing the `Scan` request.
    ///
    /// The result is a `Stream` of all ordered key value pairs within the given key range.
    pub fn scan_range(&self, start: Key, end: Key) -> impl Stream<Item = Entry, Error = Error> {
        let request = request::scan_range(self.uri.clone(), start, end);
        request_stream_and_deserialize(self, request)
    }

    /// A method for perfomring the `Max` request.
    ///
    /// The result is a `Future` yielding the greatest entry in the `sled::Tree`.
    ///
    /// Returns `None` if there are no entries within the tree.
    pub fn max(&self) -> impl Future<Item = Option<Entry>, Error = Error> {
        let request = request::max(self.uri.clone());
        request_concat_and_deserialize(self, request)
    }

    /// A method for performing the `Pred` request.
    ///
    /// Given the key for an entry in the `sled::Tree`, produce a `Future` with the preceding
    /// entry.
    pub fn pred(&self, key: Key) -> impl Future<Item = Option<Entry>, Error = Error> {
        let request = request::pred(self.uri.clone(), key);
        request_concat_and_deserialize(self, request)
    }

    /// A method for performing the `PredIncl` request.
    ///
    /// Given the key for an entry in the `sled::Tree`, produce a `Future` with the preceding
    /// entry or the entry associated with the key if there is one.
    pub fn pred_incl(&self, key: Key) -> impl Future<Item = Option<Entry>, Error = Error> {
        let request = request::pred_incl(self.uri.clone(), key);
        request_concat_and_deserialize(self, request)
    }

    /// A method for performing the `Succ` request.
    ///
    /// Given the key for an entry in the `sled::Tree`, produce a `Future` with the following
    /// entry.
    pub fn succ(&self, key: Key) -> impl Future<Item = Option<Entry>, Error = Error> {
        let request = request::succ(self.uri.clone(), key);
        request_concat_and_deserialize(self, request)
    }

    /// A method for performing the `SuccIncl` request.
    ///
    /// Given the key for an entry in the `sled::Tree`, produce a `Future` with the following
    /// entry or the entry associated with the key if there is one.
    pub fn succ_incl(&self, key: Key) -> impl Future<Item = Option<Entry>, Error = Error> {
        let request = request::succ_incl(self.uri.clone(), key);
        request_concat_and_deserialize(self, request)
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

/// Concatenate and deserialize a single-chunk reponse.
fn concat_and_deserialize<T>(response: Response<Body>) -> impl Future<Item = T, Error = Error>
where
    T: for<'de> Deserialize<'de>,
{
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
}

/// Convert the given response body chunks into a stream of deserialized items.
fn stream_and_deserialize<T>(response: Response<Body>) -> impl Stream<Item = T, Error = Error>
where
    T: for<'de> Deserialize<'de>,
{
    response
        .into_body()
        .map_err(Error::Hyper)
        .and_then(|chunk| serde_json::from_slice(&chunk).map_err(Error::SerdeJson))
}

/// Submit the given request, then concatenate and deserialize a single-chunk response.
fn request_concat_and_deserialize<T>(
    client: &Client,
    request: Request<Body>,
) -> impl Future<Item = T, Error = Error>
where
    T: for<'de> Deserialize<'de>,
{
    client
        .client
        .request(request)
        .map_err(Error::Hyper)
        .and_then(concat_and_deserialize)
}

/// Submit the given request, then convert the response body chunks into a stream of deserialized
/// items.
fn request_stream_and_deserialize<T>(
    client: &Client,
    request: Request<Body>,
) -> impl Stream<Item = T, Error = Error>
where
    T: for<'de> Deserialize<'de>,
{
    client
        .client
        .request(request)
        .map_err(Error::Hyper)
        .map(stream_and_deserialize)
        .flatten_stream()
}
