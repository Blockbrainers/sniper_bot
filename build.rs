use ethers::{prelude::Abigen};

fn main() {
    // configure the project with all its paths, solc, cache etc.
    println!("Building ethers contract");
    // let gen = MultiAbigen::from_json_files("./abi").unwrap();
    // gen.write_to_module("./src/bindings").unwrap();
    println!("cargo:rerun-if-changed=./abis/*.json");
    bindgen("Erc20");

    bindgen("UniswapV2Router02");
    bindgen("UniswapV3SmartRouter");

    bindgen("UniswapV2Factory");
    bindgen("UniswapV3Factory");

    bindgen("UniswapV2Pair");
    bindgen("UniswapV3Pool");
}

#[allow(dead_code)]
fn bindgen(fname: &str) {
    let bindings = Abigen::new(fname, format!("./abis/{}.json", fname))
        .expect("could not instantiate Abigen")
        .generate()
        .expect("could not generate bindings");

    bindings
        .write_to_file(format!("./src/bindings/{}.rs", snake_case(fname)))
        .expect("could not write bindings to file");
}

fn snake_case(name: &str) -> String {
    let mut snake_case_name = String::new();
    let mut chars = name.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() && !snake_case_name.is_empty() && chars.peek().is_some() {
            snake_case_name.push('_');
        }
        snake_case_name.push(c.to_ascii_lowercase());
    }

    snake_case_name
}