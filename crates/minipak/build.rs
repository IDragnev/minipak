use std::{
    path::{ Path, PathBuf },
    process::Command,
};

fn main() {
    println!("cargo:rustc-link-arg=-nostartfiles");
    println!("cargo:rustc-link-arg=-nodefaultlibs");
    println!("cargo:rustc-link-arg=-static");

    cargo_build(&PathBuf::from("../stage1"));
    cargo_build(&PathBuf::from("../stage2"));
}

fn cargo_build(path: &Path) {
    println!("cargo:rerun-if-changed=..");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let target_dir = format!("{}/embeds", out_dir);

    let output = Command::new("cargo")
        .arg("build")
        .arg("--target-dir")
        .arg(&target_dir)
        .arg("--release")
        .current_dir(path)
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    if output.status.success() == false {
        panic!(
            "Building {} failed.\nStoud: {}\nStderr: {}",
            path.display(),
            String::from_utf8_lossy(&output.stdout[..]),
            String::from_utf8_lossy(&output.stderr[..]),
        );
    }

    // Let's just assume the library has the same name as the crate
    let binary_name = format!("lib{}.so", path.file_name().unwrap().to_str().unwrap());
    let output = Command::new("objcopy")
        .arg("--strip-all")
        .arg(&format!("release/{}", binary_name))
        .arg(binary_name)
        .current_dir(&target_dir)
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    if output.status.success() == false {
        panic!(
            "Stripping failed.\nStdout: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout[..]),
            String::from_utf8_lossy(&output.stderr[..]),
        );
    }
}