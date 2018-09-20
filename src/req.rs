//! Functions to simplify the construction of requests along with request types that can be
//! serialized to and from the JSON body.

use hyper::{Method, Request, Uri};
use serde::Serialize;
use serde_json;

/// Types that represent a request being made to the server.
pub trait Req {
    /// The HTTP method included with the header.
    const METHOD: Method;
    /// The component of the URI following the domain.
    const URI: &'static str;
}

/// Types that may be converted into a serialized JSON body for a hyper request.
pub trait IntoBody {
    /// The body of the request, capable of being serialized to JSON.
    type Body: Serialize;
    /// Convert `self` into the serializable `Body` type.
    fn into_body(self) -> Self::Body;
}

/// Types that may be directly converted into a hyper Request.
pub trait IntoRequest: Req + IntoBody {
    fn into_request(self) -> Request<Vec<u8>>;
}

// The vector of bytes used as a key into a `sled::Tree`.
type Key = Vec<u8>;
// The vector of bytes representing a value within a `sled::Tree`.
type Value = Vec<u8>;

/// Get a single entry from the DB, identified by the given unique key.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Get {
    pub key: Key,
}

/// Delete the entry at the given key.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Del {
    pub key: Key,
}

/// Set the entry with the given key and value, replacing the original if one exists.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Set {
    pub key: Key,
    pub value: Value,
}

// /// Get the first entry that follows the given key.
// #[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
// pub struct Scan {
//     pub key: Key,
// }

// /// Scan a chunk of entries from the DB, starting from the given key.
// ///
// /// Responds with the first entry in the list, the last entry in the list, and the list of
// /// entries itself. 
// #[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
// pub struct ScanChunk {
//     pub key: Key,
// }

impl Req for Get {
    const METHOD: Method = Method::GET;
    const URI: &'static str = "/tree/entries/get";
}

impl Req for Del {
    const METHOD: Method = Method::DELETE;
    const URI: &'static str = "/tree/entries/delete";
}

impl Req for Set {
    const METHOD: Method = Method::POST;
    const URI: &'static str = "/tree/entries/set";
}

impl IntoBody for Get {
    type Body = Self;
    fn into_body(self) -> Self::Body { self }
}

impl IntoBody for Del {
    type Body = Self;
    fn into_body(self) -> Self::Body { self }
}

impl IntoBody for Set {
    type Body = Self;
    fn into_body(self) -> Self::Body { self }
}

impl<T> IntoRequest for T
where
    T: Req + IntoBody,
{
    fn into_request(self) -> Request<Vec<u8>> {
        let method = T::METHOD;
        let uri = Uri::from_static(T::URI);
        let body = self.into_body();
        let body_json = serde_json::to_vec(&body).expect("failed to serialize request body");
        Request::builder()
            .method(method)
            .uri(uri)
            .body(body_json)
            .expect("attempted to construct invalid request")
    }
}

/// A request to download the entire tree.
///
/// The body of the returned key is a `Get` serialized to JSON form.
pub fn from<T>(req: T) -> Request<Vec<u8>>
where
    T: IntoRequest,
{
    req.into_request()
}

/// Shorthand for `from(Get { key })`.
pub fn get(key: Key) -> Request<Vec<u8>> {
    from(Get { key })
}

/// Shorthand for `from(Del { key })`.
pub fn del(key: Key) -> Request<Vec<u8>> {
    from(Del { key })
}

/// Shorthand for `from(Set { key, value })`.
pub fn set(key: Key, value: Value) -> Request<Vec<u8>> {
    from(Set { key, value })
}
