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

/// The vector of bytes used as a key into a `sled::Tree`.
type Key = Vec<u8>;
/// The vector of bytes representing a value within a `sled::Tree`.
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

/// Iterate over all entries within the `Tree` within the given key range.
///
/// The given range is non-inclusive of the `end` key.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ScanRange {
    pub start: Key,
    pub end: Key,
}

/// Retrieve the entry with the greatest `Key` in the `Tree`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Max;

/// Retrieve the entry that precedes the `Key`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Pred {
    pub key: Key,
}

/// Retrieve the entry that precedes or includes the `Key`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PredIncl {
    pub key: Key,
}

/// Retrieve the entry that follows the `Key`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Succ {
    pub key: Key,
}

/// Retrieve the entry that follows or includes the `Key`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SuccIncl {
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

impl RequestType for ScanRange {
    const METHOD: Method = Method::GET;
    const PATH_AND_QUERY: &'static str = "/tree/entries/scan_range";
}

impl RequestType for Max {
    const METHOD: Method = Method::GET;
    const PATH_AND_QUERY: &'static str = "/tree/entries/max";
}

impl RequestType for Pred {
    const METHOD: Method = Method::GET;
    const PATH_AND_QUERY: &'static str = "/tree/entries/pred";
}

impl RequestType for PredIncl {
    const METHOD: Method = Method::GET;
    const PATH_AND_QUERY: &'static str = "/tree/entries/pred_incl";
}

impl RequestType for Succ {
    const METHOD: Method = Method::GET;
    const PATH_AND_QUERY: &'static str = "/tree/entries/succ";
}

impl RequestType for SuccIncl {
    const METHOD: Method = Method::GET;
    const PATH_AND_QUERY: &'static str = "/tree/entries/succ_incl";
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

impl IntoBody for ScanRange {
    type Body = Self;
    fn into_body(self) -> Self::Body { self }
}

impl IntoBody for Max {
    type Body = Self;
    fn into_body(self) -> Self::Body { self }
}

impl IntoBody for Pred {
    type Body = Self;
    fn into_body(self) -> Self::Body { self }
}

impl IntoBody for PredIncl {
    type Body = Self;
    fn into_body(self) -> Self::Body { self }
}

impl IntoBody for Succ {
    type Body = Self;
    fn into_body(self) -> Self::Body { self }
}

impl IntoBody for SuccIncl {
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

/// Shorthand for `from(base_uri, Get { key })`.
pub fn get(base_uri: Uri, key: Key) -> Request<Body> {
    from(base_uri, Get { key })
}

/// Shorthand for `from(base_uri, Del { key })`.
pub fn del(base_uri: Uri, key: Key) -> Request<Body> {
    from(base_uri, Del { key })
}

/// Shorthand for `from(base_uri, Set { key, value })`.
pub fn set(base_uri: Uri, key: Key, value: Value) -> Request<Body> {
    from(base_uri, Set { key, value })
}

/// Shorthand for `from(base_uri, Iter)`.
pub fn iter(base_uri: Uri) -> Request<Body> {
    from(base_uri, Iter)
}

/// Shorthand for `from(base_uri, Scan { key })`.
pub fn scan(base_uri: Uri, key: Key) -> Request<Body> {
    from(base_uri, Scan { key })
}

/// Shorthand for `from(base_uri, ScanRange { start, end })`.
pub fn scan_range(base_uri: Uri, start: Key, end: Key) -> Request<Body> {
    from(base_uri, ScanRange { start, end })
}

/// Shorthand for `from(base_uri, Max)`.
pub fn max(base_uri: Uri) -> Request<Body> {
    from(base_uri, Max)
}

/// Shorthand for `from(base_uri, Pred { key })`.
pub fn pred(base_uri: Uri, key: Key) -> Request<Body> {
    from(base_uri, Pred { key })
}

/// Shorthand for `from(base_uri, PredIncl { key })`.
pub fn pred_incl(base_uri: Uri, key: Key) -> Request<Body> {
    from(base_uri, PredIncl { key })
}

/// Shorthand for `from(base_uri, Succ { key })`.
pub fn succ(base_uri: Uri, key: Key) -> Request<Body> {
    from(base_uri, Succ { key })
}

/// Shorthand for `from(base_uri, SuccIncl { key })`.
pub fn succ_incl(base_uri: Uri, key: Key) -> Request<Body> {
    from(base_uri, SuccIncl { key })
}
