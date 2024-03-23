use std::future::Future;
fn main() {
    // regular closure
    let f1 = || "hello world".split(' ').collect::<Vec<&str>>();
    println!("f1: {:#?}", f1());
    // async closure
    let f2 = async || "hello world".split(' ').collect::<Vec<&str>>();
}
