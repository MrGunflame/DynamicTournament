use dynamic_tournament_web::{run_with_config, Config};

fn main() {
    run_with_config(Config {
        api_base: "http://localhost:3030".into(),
        root: "/".into(),
        mountpoint: "main".into(),
    });
}
