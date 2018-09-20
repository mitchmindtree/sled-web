extern crate sled_web;

use sled_web::hyper::{self, Client, Uri};
use sled_web::hyper::rt::{Future, Stream};
use std::io::{self, Write};

fn main() {
    hyper::rt::run(get_elem(vec![2]));
}

fn get_elem(key: Vec<u8>) -> impl Future<Item=(), Error=()> {
    let client = Client::new();
    let uri = "http://localhost:3000/".parse::<Uri>().unwrap();
    client
        .get(uri)
        .and_then(|res| {
            res.into_body().for_each(|chunk| {
                io::stdout()
                    .write_all(&chunk)
                    .map_err(|e| panic!("error writing to stdio: {}", e))
            })
        })
        .map(|res| println!("Success: {:#?}", res))
        .map_err(|err| eprintln!("Error: {}", err))
}
