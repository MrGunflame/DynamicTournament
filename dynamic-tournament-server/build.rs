use std::path::Path;
use std::process::Command;

const SECRET_PATH: &str = "jwt-secret";

fn main() {
    println!("cargo:rerun-if-changed=jwt-secret");

    if !Path::new(SECRET_PATH).exists() {
        Command::new("dd")
            .args(&["if=/dev/urandom", "of=./jwt-secret", "bs=1", "count=512"])
            .spawn()
            .unwrap();
    }
}
