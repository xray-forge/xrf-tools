//! Points the linker at SDL's import library: `sdl3-sys` links `SDL3` with no search path where neither pkg-config nor
//! vcpkg answers, and `.cargo/config.toml` sets `SDL3_LIB_DIR` to the copy `cargo make fetch-sdks` places.

fn main() {
  println!("cargo::rerun-if-env-changed=SDL3_LIB_DIR");

  if let Ok(directory) = std::env::var("SDL3_LIB_DIR") {
    println!("cargo::rustc-link-search=native={directory}");
  }
}
