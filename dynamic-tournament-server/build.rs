use std::path::Path;
use std::process::Command;

const SECRET_PATH: &str = "jwt-secret";

fn main() {
    println!("cargo:rerun-if-changed={}", SECRET_PATH);

    if !Path::new(SECRET_PATH).exists() {
        Command::new("dd")
            .args(&[
                "if=/dev/urandom",
                &format!("of={}", SECRET_PATH),
                "bs=1",
                "count=512",
            ])
            .spawn()
            .unwrap();
    }
}
