//! Functions to simplify the construction of requests along with request types that can be
//! serialized to and from the JSON body.

use http::uri::PathAndQuery;
use hyper::{Body, Method, Request, Uri};
use serde::Serialize;
use serde_json;

/// Types that represent a request being made to the server.
pub trait RequestType {
    /// The HTTP method included with the header.
    const METHOD: Method;
    /// The component of the URI following the domain.
    const PATH_AND_QUERY: &'static str;
}

/// Types that may be converted into a serialized JSON body for a hyper request.
pub trait IntoBody {
    /// The body of the request, capable of being serialized to JSON.
    type Body: Serialize;
    /// Convert `self` into the serializable `Body` type.
    fn into_body(self) -> Self::Body;
}

/// Types that may be directly converted into a hyper Request.
pub trait IntoRequest: RequestType + IntoBody {
    /// The `base_uri` should include only the scheme and host - the path and query will be
    /// retrieved via `RequestType::PATH_AND_QUERY`.
    fn into_request(self, base_uri: Uri) -> Request<Body>;
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

/// Iterate over all entries within the `Tree`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Iter;

/// Iterate over all entries within the `Tree` that start at or follow the given key.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Scan {
    pub key: Key,
}

impl RequestType for Get {
    const METHOD: Method = Method::GET;
    const PATH_AND_QUERY: &'static str = "/tree/entries/get";
}

impl RequestType for Del {
    const METHOD: Method = Method::DELETE;
    const PATH_AND_QUERY: &'static str = "/tree/entries/delete";
}

impl RequestType for Set {
    const METHOD: Method = Method::POST;
    const PATH_AND_QUERY: &'static str = "/tree/entries/set";
}

impl RequestType for Iter {
    const METHOD: Method = Method::GET;
    const PATH_AND_QUERY: &'static str = "/tree/entries/iter";
}

impl RequestType for Scan {
    const METHOD: Method = Method::GET;
    const PATH_AND_QUERY: &'static str = "/tree/entries/scan";
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

impl IntoBody for Iter {
    type Body = Self;
    fn into_body(self) -> Self::Body { self }
}

impl IntoBody for Scan {
    type Body = Self;
    fn into_body(self) -> Self::Body { self }
}

impl<T> IntoRequest for T
where
    T: RequestType + IntoBody,
{
    fn into_request(self, base_uri: Uri) -> Request<Body> {
        let method = T::METHOD;
        let uri = uri_with_path(base_uri, T::PATH_AND_QUERY);
        let body = self.into_body();
        let body_json = serde_json::to_vec(&body).expect("failed to serialize request body");
        Request::builder()
            .method(method)
            .uri(uri)
            .body(body_json.into())
            .expect("attempted to construct invalid request")
    }
}

/// Append the given path to the given `Uri`.
///
/// Assumes the `Uri` already contains the scheme and authority parts.
fn uri_with_path(uri: Uri, path: &str) -> Uri {
    let mut parts = uri.into_parts();
    let path_and_query = path
        .parse::<PathAndQuery>()
        .expect("failed to parse path and query for request URI");
    parts.path_and_query = Some(path_and_query);
    Uri::from_parts(parts)
        .expect("failed to construct request URI from parts")
}

/// A request to download the entire tree.
///
/// The body of the returned key is a `Get` serialized to JSON form.
pub fn from<T>(base_uri: Uri, req: T) -> Request<Body>
where
    T: IntoRequest,
{
    req.into_request(base_uri)
}

/// Shorthand for `from(Get { key })`.
pub fn get(base_uri: Uri, key: Key) -> Request<Body> {
    from(base_uri, Get { key })
}

/// Shorthand for `from(Del { key })`.
pub fn del(base_uri: Uri, key: Key) -> Request<Body> {
    from(base_uri, Del { key })
}

/// Shorthand for `from(Set { key, value })`.
pub fn set(base_uri: Uri, key: Key, value: Value) -> Request<Body> {
    from(base_uri, Set { key, value })
}

/// Shorthand for `from(Iter)`.
pub fn iter(base_uri: Uri) -> Request<Body> {
    from(base_uri, Iter)
}

/// Shorthand for `from(Scan { key })`.
pub fn scan(base_uri: Uri, key: Key) -> Request<Body> {
    from(base_uri, Scan { key })
}
