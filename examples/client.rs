extern crate sled_web;
extern crate serde_json;

use sled_web::hyper;
use sled_web::hyper::rt::{Future, Stream};

fn main() {
    let client = sled_web::Client::new("http://localhost:3000".parse().unwrap());

    let get_a = client
        .get(vec![6])
        .map(|v| {
            println!("Entry `6` was `None`, as expected");
            assert!(v.is_none())
        })
        .map_err(|e| eprintln!("{}", e));

    let set = client
        .set(vec![6], vec![1, 2, 3, 4, 5, 6, 7, 8, 9])
        .map(|_| println!("Entry `6` successfully set"))
        .map_err(|e| eprintln!("{}", e));

    let get_b = client
        .get(vec![6])
        .map(|v| {
            assert!(v.is_some());
            println!("Successfully retrieved `6`: {:?}", v)
        })
        .map_err(|e| eprintln!("{}", e));

    let iter = client
        .iter()
        .map(|(k, v)| println!("  ({:?}, {:?})", k, v))
        .map_err(|e| eprintln!("Error: {}", e))
        .collect()
        .map(|_| ())
        .map_err(|_| ());

    let scan = client
        .scan(vec![3])
        .map(|(k, v)| println!("  ({:?}, {:?})", k, v))
        .map_err(|e| eprintln!("Error: {}", e))
        .collect()
        .map(|_| ())
        .map_err(|_| ());

    let scan_range = client
        .scan_range(vec![2], vec![5])
        .map(|(k, v)| println!("  ({:?}, {:?})", k, v))
        .map_err(|e| eprintln!("Error: {}", e))
        .collect()
        .map(|_| ())
        .map_err(|_| ());

    let max = client
        .max()
        .map(|entry| println!("Max: {:?}", entry))
        .map_err(|e| eprintln!("Error: {}", e));

    let pred = client
        .pred(vec![4])
        .map(|entry| println!("Pred to [4]: {:?}", entry))
        .map_err(|e| eprintln!("Error: {}", e));

    let pred_incl = client
        .pred_incl(vec![4])
        .map(|entry| println!("PredIncl to [4]: {:?}", entry))
        .map_err(|e| eprintln!("Error: {}", e));

    let succ = client
        .succ(vec![2])
        .map(|entry| println!("Succ to [2]: {:?}", entry))
        .map_err(|e| eprintln!("Error: {}", e));

    let succ_incl = client
        .succ_incl(vec![2])
        .map(|entry| println!("SuccIncl to [2]: {:?}", entry))
        .map_err(|e| eprintln!("Error: {}", e));

    hyper::rt::run({
        get_a
            .then(|_| set)
            .then(|_| get_b)
            .then(|_| {
                println!("Iter all elements...");
                iter
            })
            .then(|_| {
                println!("Scan elements starting from `vec![3]`");
                scan
            })
            .then(|_| {
                println!("Scan range elements starting from `vec![2]` to `vec![5]`");
                scan_range
            })
            .then(|_| max)
            .then(|_| pred)
            .then(|_| pred_incl)
            .then(|_| succ)
            .then(|_| succ_incl)
    });
}
