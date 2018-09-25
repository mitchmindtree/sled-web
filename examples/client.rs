extern crate sled_web;
extern crate serde_json;

use sled_web::hyper::{self, Client, Request, Uri};
use sled_web::hyper::rt::{Future, Stream};
use std::io::{self, Write};

fn main() {
    let client = sled_web::Client::new("http://localhost:3000".parse().unwrap());

    let a = client
        .get(vec![6])
        .map(|v| {
            println!("Entry `6` was `None`, as expected");
            assert!(v.is_none())
        })
        .map_err(|e| eprintln!("{}", e));

    let b = client
        .set(vec![6], vec![1, 2, 3, 4, 5, 6, 7, 8, 9])
        .map(|_| println!("Entry `6` successfully set"))
        .map_err(|e| eprintln!("{}", e));

    let c = client
        .get(vec![6])
        .map(|v| {
            assert!(v.is_some());
            println!("Successfully retrieved `6`: {:?}", v)
        })
        .map_err(|e| eprintln!("{}", e));

    let iter = client
        .iter()
        .map(|(k, v)| println!("Key: {:?}, Value: {:?}", k, v))
        .map_err(|e| eprintln!("Error: {}", e))
        .collect()
        .map(|_| ())
        .map_err(|_| ());

    let scan = client
        .scan(vec![3])
        .map(|(k, v)| println!("Key: {:?}, Value: {:?}", k, v))
        .map_err(|e| eprintln!("Error: {}", e))
        .collect()
        .map(|_| ())
        .map_err(|_| ());

    hyper::rt::run({
        a.then(|_| b)
            .then(|_| c)
            .then(|_| {
                println!("Iter all elements...");
                iter
            })
            .then(|_| {
                println!("Scan elements starting from `vec![3]`");
                scan
            })
    });
}
