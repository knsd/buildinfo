#[macro_use] extern crate buildinfo;

fn main() {
    let info = buildinfo!();
    println!("{:?}", info)
}
