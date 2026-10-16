use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-env-changed=GIT_COMMIT_SHA");

    let commit_sha = std::env::var("GIT_COMMIT_SHA").ok().or_else(|| {
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    });

    if let Some(sha) = commit_sha {
        if !sha.is_empty() {
            println!("cargo:rustc-env=GIT_COMMIT_SHA={}", sha);
        }
    }
}
