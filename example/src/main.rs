#[macro_use] extern crate buildinfo;

fn main() {
    let info = buildinfo!();

    println!("Target triple: {}", info.target_triple());
    println!("Host triple: {}", info.host_triple());
    println!("Opt level: {}", info.opt_level());
    println!("Debug: {}", info.debug());
    println!("Profile: {}", info.profile());
    println!("Rustc version: {}", info.rustc_version());
    println!("Compiled at: {:?}", info.compiled_at());
    println!("Git commit: {:?}", info.git_commit());
    println!("Hostname: {:?}", info.hostname());
    println!("Username: {:?}", info.username());

    println!("{:?}", info)
}
