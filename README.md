# sled-web [![Build Status](https://travis-ci.org/mitchmindtree/sled-web.svg?branch=master)](https://travis-ci.org/mitchmindtree/sled-web) [![Crates.io](https://img.shields.io/crates/v/sled-web.svg)](https://crates.io/crates/sled-web) [![Crates.io](https://img.shields.io/crates/l/sled-web.svg)](https://github.com/mitchmindtree/sled-web/blob/master/LICENSE-MIT) [![docs.rs](https://docs.rs/sled-web/badge.svg)](https://docs.rs/sled-web/)

An extension of the `sled` crate that allows for accessing a `sled::Tree` via a
client/server API using the `hyper` web framework crate.

## Client API

| HTTP Request                      | Description
|-----------------------------------|--------------------------------------
| GET    `/tree/entries/get`        | Get a `Tree` entry by key.
| DELETE `/tree/entries/del`        | Delete a `Tree` entry by key.
| POST   `/tree/entries/set`        | Set a new `Tree` entry by key/value pair.
| PUT    `/tree/entries/cas`        | Perform a compare-and-swap.
| POST   `/tree/entries/merge`      | Merge a value into an entry for a key.
| POST   `/tree/entries/flush`      | Flush and pending IO.
| GET    `/tree/entries/iter`       | Iterate over all `Tree` entries.
| GET    `/tree/entries/scan`       | Iterate over all `Tree` entries starting from a key.
| GET    `/tree/entries/scan_range` | Iterate over all `Tree` entries within a key range.
| GET    `/tree/entries/max`        | Get the greatest `Tree` entry.
| GET    `/tree/entries/pred`       | Get the `Tree` entry preceding a key.
| GET    `/tree/entries/pred_incl`  | Get the `Tree` entry preceding or including a key.
| GET    `/tree/entries/succ`       | Get the `Tree` entry succeeding a key.
| GET    `/tree/entries/succ_incl`  | Get the `Tree` entry succeeding or including a key.

See the `request` module for the expected request types. The server expects the
corresponding request type serialized to JSON within the `Body` of the received
`Request`.

See the `response::response` function for the associated responses, their status
and layout.
