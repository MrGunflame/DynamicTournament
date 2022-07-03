use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use rand::rngs::OsRng;
use rand::RngCore;

const SECRET_PATH: &str = "jwt-secret";

fn main() {
    println!("cargo:rerun-if-changed={}", SECRET_PATH);

    if !Path::new(SECRET_PATH).exists() {
        let mut buf = [0u8; 512];
        OsRng.fill_bytes(&mut buf);

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(SECRET_PATH)
            .unwrap();

        file.write_all(&buf).unwrap();
    }
}
