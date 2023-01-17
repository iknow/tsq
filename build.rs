use std::env;

fn main() {
    let grammars = env::var("GRAMMARS").expect("grammar path provided in environment");
    println!("cargo:rustc-link-search={}/lib", grammars);
    println!("cargo:rustc-link-lib=typescript");
    println!("cargo:rustc-link-lib=tsx");
}
