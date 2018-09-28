extern crate sled_web;

use sled_web::sled;

fn main() {
    let tree = sled::Tree::start(sled::ConfigBuilder::new().temporary(true).build()).unwrap();
    tree.set(vec![1], vec![1, 2, 3, 4]).unwrap();
    tree.set(vec![2], vec![5, 6, 7, 8]).unwrap();
    tree.set(vec![4], vec![1, 2, 4, 8]).unwrap();
    let config = sled_web::server::config()
        .addr(([127, 0, 0, 1], 3000))
        .build();
    sled_web::server::run(config, std::sync::Arc::new(tree));
}
