//! A web interface to a `sled::Tree`.
//!
//! ## API
//!
//! REST:
//!
//!          | HTTP Request                              | Description
//! ---------|-------------------------------------------|--------------------------------------
//! Tree     |                                           |
//!          | GET    /tree                              | whole `Tree`.
//!          | GET    /tree/iter                         | `iter` of `Tree` entries.
//!          | GET    /tree/scan/:key                    | `scan` of `Tree` entries.
//!          | GET    /tree/entries/:key                 | a specific `Tree` entry by key.
//!          | DELETE /tree/entries/:key                 | a `Tree` entry by key.
//!          | PUT    /tree/entries/:key_value           | a new `Tree` entry by key/value pair.
//!
//! The following should not be implemented, however there should be something similar implemented
//! in the API that translates each of these into their raw `Tree` calls as above.
//!
//! ---------|-------------------------------------------|--------------------------------------
//! Table    |                                           |
//!          | GET    /tree/:table                       | whole `Table`.
//!          | GET    /tree/:table/iter                  | `iter` of `Table` entries.
//!          | GET    /tree/:table/scan/:key             | `scan` of `Table` entries.
//!          | GET    /tree/:table/entries/:key          | a specific `Table` entry by key.
//!          | DELETE /tree/:table/entries/:key          | a `Table` entry by key.
//!          | PUT    /tree/:table/entries/:key_value    | a new `Table` entry by key/value pair.

#[macro_use] extern crate serde_derive;
extern crate futures;
extern crate http;
extern crate serde;
extern crate serde_json;
pub extern crate hyper;
pub extern crate sled_table;

pub use client::Client;
pub use sled_table::sled;

pub mod client;
pub mod request;
pub mod response;
pub mod server;
