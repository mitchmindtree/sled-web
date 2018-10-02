use futures;
use hyper::{self, Body, Chunk, Request, Response, StatusCode};
use hyper::rt::{Future, Stream};
use request::{self, RequestType};
use serde::Deserialize;
use serde_json;
use sled;
use sled_search;
use std::error::Error as StdError;
use std::mem;
use std::sync::Arc;

/// Types that may be produced in response to some request.
pub trait IntoResponse {
    /// Respond to the given request body, updating the `sled::Tree` as necessary.
    fn into_response(self, Arc<sled::Tree>) -> Response<Body>;
}

/// A response to some request wrapped in a `Future`.
pub type ResponseFuture = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

/// A wrapper around the `sled::Tree` iterator which is `'static`.
///
/// This is achieved by unsafely transmuting the lifetime of the iterator to `'static`. We can do
/// so safely by ensuring that the `Arc<Tree>` to which the original lifetime was bound is stored
/// alongside the iterator itself, guaranteeing that the `Tree` live at least as long as this
/// `Iter`.
struct Iter {
    _tree: Arc<sled::Tree>,
    iter: sled::Iter<'static>,
}

impl IntoResponse for request::Get {
    fn into_response(self, tree: Arc<sled::Tree>) -> Response<Body> {
        tree.get(&self.key)
            .map(|value| {
                let bytes = serde_json::to_vec(&value)
                    .expect("failed to serialize value to JSON");
                Response::new(bytes.into())
            })
            .unwrap_or_else(|err| db_err_response(&err))
    }
}

impl IntoResponse for request::Del {
    fn into_response(self, tree: Arc<sled::Tree>) -> Response<Body> {
        tree.del(&self.key)
            .map(|value| {
                let bytes = serde_json::to_vec(&value)
                    .expect("failed to serialize value to JSON");
                Response::new(bytes.into())
            })
            .unwrap_or_else(|err| db_err_response(&err))
    }
}

impl IntoResponse for request::Set {
    fn into_response(self, tree: Arc<sled::Tree>) -> Response<Body> {
        let request::Set { key, value } = self;
        tree.set(key, value)
            .map(|value| {
                let bytes = serde_json::to_vec(&value)
                    .expect("failed to serialize value to JSON");
                Response::builder()
                    .status(StatusCode::CREATED)
                    .body(bytes.into())
                    .expect("failed to construct `Set` response")
            })
            .unwrap_or_else(|err| db_err_response(&err))
    }
}

impl IntoResponse for request::Cas {
    fn into_response(self, tree: Arc<sled::Tree>) -> Response<Body> {
        let request::Cas { key, old, new } = self;
        match tree.cas(key, old, new) {
            Ok(()) => {
                let res: Result<(), Option<Vec<u8>>> = Ok(());
                let bytes = serde_json::to_vec(&res)
                    .expect("failed to serialize result to JSON");
                Response::new(bytes.into())
            }
            Err(sled::Error::CasFailed(opt_bytes)) => {
                let res: Result<(), Option<Vec<u8>>> = Err(opt_bytes);
                let bytes = serde_json::to_vec(&res)
                    .expect("failed to serialize result to JSON");
                Response::new(bytes.into())
            }
            Err(err) => db_err_response(&err),
        }
    }
}

impl IntoResponse for request::Merge {
    fn into_response(self, tree: Arc<sled::Tree>) -> Response<Body> {
        let request::Merge { key, value } = self;
        tree.merge(key, value)
            .map(|value| {
                let bytes = serde_json::to_vec(&value)
                    .expect("failed to serialize value to JSON");
                Response::builder()
                    .status(StatusCode::CREATED)
                    .body(bytes.into())
                    .expect("failed to construct `Set` response")
            })
            .unwrap_or_else(|err| db_err_response(&err))
    }
}

impl IntoResponse for request::Flush {
    fn into_response(self, tree: Arc<sled::Tree>) -> Response<Body> {
        tree.flush()
            .map(|value| {
                let bytes = serde_json::to_vec(&value)
                    .expect("failed to serialize value to JSON");
                Response::new(bytes.into())
            })
            .unwrap_or_else(|err| db_err_response(&err))
    }
}

impl IntoResponse for request::Iter {
    fn into_response(self, tree: Arc<sled::Tree>) -> Response<Body> {
        let iter = tree_iter(tree)
            .map(|res| {
                let kv = res.map_err(|err| Box::new(err))?;
                let bytes = serde_json::to_vec(&kv).map_err(|err| Box::new(err))?;
                Ok(Chunk::from(bytes))
            });
        let stream = Box::new(futures::stream::iter_result(iter)) as Box<_>;
        Response::builder()
            .body(Body::from(stream))
            .expect("failed to construct `Iter` response")
    }
}

impl IntoResponse for request::Scan {
    fn into_response(self, tree: Arc<sled::Tree>) -> Response<Body> {
        let scan = tree_scan(tree, &self.key)
            .map(|res| {
                let kv = res.map_err(|err| Box::new(err))?;
                let bytes = serde_json::to_vec(&kv).map_err(|err| Box::new(err))?;
                Ok(Chunk::from(bytes))
            });
        let stream = Box::new(futures::stream::iter_result(scan)) as Box<_>;
        Response::builder()
            .body(Body::from(stream))
            .expect("failed to construct `Iter` response")
    }
}

impl IntoResponse for request::ScanRange {
    fn into_response(self, tree: Arc<sled::Tree>) -> Response<Body> {
        let request::ScanRange { start, end } = self;
        let scan = tree_scan(tree, &start)
            .filter_map(move |res| {
                let (k, v) = match res {
                    Err(err) => return Some(Err(Box::new(err) as Box<StdError + Send + Sync>)),
                    Ok(kv) => kv,
                };
                if k >= end {
                    return None;
                }
                let bytes = match serde_json::to_vec(&(k, v)) {
                    Err(err) => return Some(Err(Box::new(err))),
                    Ok(bytes) => bytes,
                };
                Some(Ok(Chunk::from(bytes)))
            });
        let stream = Box::new(futures::stream::iter_result(scan)) as Box<_>;
        Response::builder()
            .body(Body::from(stream))
            .expect("failed to construct `Iter` response")
    }
}

impl IntoResponse for request::Max {
    fn into_response(self, tree: Arc<sled::Tree>) -> Response<Body> {
        sled_search::max(&tree)
            .map(|entry| {
                let bytes = serde_json::to_vec(&entry)
                    .expect("failed to serialize entry to JSON");
                Response::builder()
                    .body(bytes.into())
                    .expect("failed to construct `Max` response")
            })
            .unwrap_or_else(|err| db_err_response(&err))
    }
}

impl IntoResponse for request::Pred {
    fn into_response(self, tree: Arc<sled::Tree>) -> Response<Body> {
        sled_search::pred(&tree, &self.key)
            .map(|entry| {
                let bytes = serde_json::to_vec(&entry)
                    .expect("failed to serialize entry to JSON");
                Response::new(bytes.into())
            })
            .unwrap_or_else(|err| db_err_response(&err))
    }
}

impl IntoResponse for request::PredIncl {
    fn into_response(self, tree: Arc<sled::Tree>) -> Response<Body> {
        sled_search::pred_incl(&tree, &self.key)
            .map(|entry| {
                let bytes = serde_json::to_vec(&entry)
                    .expect("failed to serialize entry to JSON");
                Response::new(bytes.into())
            })
            .unwrap_or_else(|err| db_err_response(&err))
    }
}

impl IntoResponse for request::Succ {
    fn into_response(mut self, tree: Arc<sled::Tree>) -> Response<Body> {
        self.key.push(0);
        let entry = match tree.scan(&self.key).next() {
            Some(Err(err)) => return db_err_response(&err),
            Some(Ok(entry)) => Some(entry),
            None => None,
        };
        let bytes = serde_json::to_vec(&entry)
            .expect("failed to serialize entry to JSON");
        Response::new(bytes.into())
    }
}

impl IntoResponse for request::SuccIncl {
    fn into_response(self, tree: Arc<sled::Tree>) -> Response<Body> {
        let entry = match tree.scan(&self.key).next() {
            Some(Err(err)) => return db_err_response(&err),
            Some(Ok(entry)) => Some(entry),
            None => None,
        };
        let bytes = serde_json::to_vec(&entry)
            .expect("failed to serialize entry to JSON");
        Response::new(bytes.into())
    }
}

impl Iterator for Iter {
    type Item = sled::Result<(Vec<u8>, Vec<u8>), ()>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// Produce an iterator over all elements within the given `Tree` with a static lifetime.
fn tree_iter(tree: Arc<sled::Tree>) -> Iter {
    let _tree = tree.clone();
    let iter: sled::Iter = tree.iter();
    let iter: sled::Iter<'static> = unsafe { mem::transmute(iter) };
    Iter { _tree, iter }
}

/// Produce a `scan` iterator over all elements within the given `Tree` with a static lifetime.
fn tree_scan(tree: Arc<sled::Tree>, key: &[u8]) -> Iter {
    let _tree = tree.clone();
    let iter: sled::Iter = tree.scan(key);
    let iter: sled::Iter<'static> = unsafe { mem::transmute(iter) };
    Iter { _tree, iter }
}

/// Deserialize a request of type `T` and produce a response.
fn deserialize_and_respond<T>(bytes: &[u8], tree: Arc<sled::Tree>) -> Response<Body>
where
    T: IntoResponse + for<'de> Deserialize<'de>,
{
    serde_json::from_slice(bytes)
        .map(|req: T| req.into_response(tree))
        .unwrap_or_else(|err| deserialization_err_response(&err))
}

/// Concatenate the given request body into a request of type `T` and produce a response.
fn concat_and_respond<T>(
    request: Request<Body>,
    tree: Arc<sled::Tree>,
) -> impl Future<Item = Response<Body>, Error = hyper::Error> + Send
where
    T: IntoResponse + for<'de> Deserialize<'de>,
{
    request
        .into_body()
        .concat2()
        .map(move |chunk| deserialize_and_respond::<T>(&chunk, tree))
}

/// Convert an error into a JSON string.
fn err_to_json_bytes(err: &StdError) -> Vec<u8> {
    let string = format!("{}", err);
    serde_json::to_vec(&string)
        .expect("failed to serialize error string")
}

/// A response to a request that resulted in a sled DB error of some kind.
///
/// Status: INTERNAL_SERVER_ERROR
/// Body: `String` of error description.
fn db_err_response(err: &StdError) -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(err_to_json_bytes(err).into())
        .expect("failed to construct INTERNAL_SERVER_ERROR response")
}

/// A response to a request that could not be successfully deserialized.
///
/// Status: BAD_REQUEST
/// Body: `String` of error description.
fn deserialization_err_response(err: &StdError) -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(err_to_json_bytes(err).into())
        .expect("failed to construct BAD_REQUEST response")
}

/// Create a response to the given request.
///
/// All response bodies will be serialized to JSON bytes.
///
/// | **Description**                   | **Status**        | **Body**                          |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::get` returns `Ok`          | 200 OK            | `Option<Vec<u8>>`                 |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::del` returns `Ok`          | 200 OK            | `Option<Vec<u8>>`                 |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::set` returns `Ok`          | 201 Created       | `()`                              |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::cas` returns `Ok`          | 200 Ok            | `Ok(())`                          |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::cas` returns `CasFailed`   | 200 Ok            | `Err(Vec<u8>)`                    |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::merge` returns `Ok`        | 200 Ok            | `()`                              |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::flush` returns `Ok`        | 200 Ok            | `()`                              |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::iter`                      | 200 OK            | Stream of `(Vec<u8>, Vec<u8>)`    |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::scan`                      | 200 OK            | Stream of `(Vec<u8>, Vec<u8>)`    |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::scan_range`                | 200 OK            | Stream of `(Vec<u8>, Vec<u8>)`    |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::pred` returns `Ok`         | 200 OK            | `Option<(Vec<u8>, Vec<u8>)>`      |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::pred_incl` returns `Ok`    | 200 OK            | `Option<(Vec<u8>, Vec<u8>)>`      |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::succ` returns `Ok`         | 200 OK            | `Option<(Vec<u8>, Vec<u8>)>`      |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `Tree::succ_incl` returns `Ok`    | 200 OK            | `Option<(Vec<u8>, Vec<u8>)>`      |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | Deserialization Errors            | 400 Bad Request   | `String`                          |
/// | --------------------------------- | ----------------- | --------------------------------- |
/// | `sled::DbResult` `Err`s           | 500 Server Error  | `String`                          |
/// | --------------------------------- | ----------------- | --------------------------------- |
pub fn response(request: Request<Body>, tree: Arc<sled::Tree>) -> ResponseFuture {
    match (request.method(), request.uri().path()) {
        (&request::Get::METHOD, request::Get::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::Get>(request, tree))
        }
        (&request::Del::METHOD, request::Del::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::Del>(request, tree))
        }
        (&request::Set::METHOD, request::Set::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::Set>(request, tree))
        }
        (&request::Cas::METHOD, request::Cas::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::Cas>(request, tree))
        }
        (&request::Merge::METHOD, request::Merge::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::Merge>(request, tree))
        }
        (&request::Flush::METHOD, request::Flush::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::Flush>(request, tree))
        }
        (&request::Iter::METHOD, request::Iter::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::Iter>(request, tree))
        }
        (&request::Scan::METHOD, request::Scan::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::Scan>(request, tree))
        }
        (&request::ScanRange::METHOD, request::ScanRange::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::ScanRange>(request, tree))
        }
        (&request::Max::METHOD, request::Max::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::Max>(request, tree))
        }
        (&request::Pred::METHOD, request::Pred::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::Pred>(request, tree))
        }
        (&request::PredIncl::METHOD, request::PredIncl::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::PredIncl>(request, tree))
        }
        (&request::Succ::METHOD, request::Succ::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::Succ>(request, tree))
        }
        (&request::SuccIncl::METHOD, request::SuccIncl::PATH_AND_QUERY) => {
            Box::new(concat_and_respond::<request::SuccIncl>(request, tree))
        }
        _ => unimplemented!()
    }
}
