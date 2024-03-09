fn main() {
    println!("cargo:rustc-link-arg=-nostartfiles");
    println!("cargo:rustc-link-arg=-nodefaultlibs");
    println!("cargo:rustc-link-arg=-static");

    cargo_build("../stage1");
}

fn cargo_build(path: &str) {
    use std::process::Command;

    println!("cargo:rerun-if-changed={}", path);

    let target_dir = format!("{}/embeds", std::env::var("OUT_DIR").unwrap());

    let output = Command::new("cargo")
        .arg("build")
        .arg("--target-dir")
        .arg(target_dir)
        .arg("--release")
        .current_dir(path)
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    if output.status.success() == false {
        panic!(
            "Building {} failed.\nStoud: {}\nStderr: {}",
            path,
            String::from_utf8_lossy(&output.stdout[..]),
            String::from_utf8_lossy(&output.stderr[..]),
        );
    }
}