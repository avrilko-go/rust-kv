use std::process::Command;

fn main() {
    let build_proto = option_env!("BUILD_PROTO")
        .map(|v| v == "1")
        .unwrap_or(false);
    if !build_proto {
        println!("=== Skipped compiling protos ===");
        return;
    }

    let mut config = prost_build::Config::new();
    config.bytes(&["."]);
    config.type_attribute(".", "#[derive(PartialOrd)]");
    config
        .out_dir("src/pb")
        .compile_protos(&["abi.proto"], &["."])
        .unwrap();
    Command::new("cargo")
        .args(["fmt", "--", "src/*.rs"])
        .status()
        .expect("cargo fmt failed");
}
