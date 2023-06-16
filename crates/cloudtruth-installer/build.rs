fn main() {
    // Make TARGET environment variable available to code at build-time
    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap()
    );
}
