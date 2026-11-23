//! Build script.
//!
//! The `sfml` crate links SFML statically. On macOS that needs the `-ObjC` linker flag,
//! otherwise SFML's Objective-C classes are left out and opening a window crashes with
//! "unrecognized selector sent to instance" (see <https://github.com/SFML/SFML/issues/2920>).

fn main() {
    // `CARGO_CFG_TARGET_OS` is the system we build FOR, so this does nothing on Linux or Windows.
    if std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "macos") {
        println!("cargo:rustc-link-arg=-ObjC");
    }
}