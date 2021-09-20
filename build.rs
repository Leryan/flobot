use std::process::Command;

fn main() {
    let mut git_hash = String::from("UNKNOWN");
    // https://stackoverflow.com/questions/43753491/include-git-commit-hash-as-string-into-rust-program
    // yup. i'm lazy. but discovered build.rs, noice
    if let Ok(output) = Command::new("git").args(&["rev-parse", "HEAD"]).output() {
        git_hash = String::from_utf8(output.stdout).unwrap();
    }
    println!("cargo:rustc-env=BUILD_GIT_HASH={}", git_hash);
}
