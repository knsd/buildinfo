#[macro_use] extern crate buildinfo;

fn main() {
    let info = build_info!();
    println!("{:?}", info)
}
