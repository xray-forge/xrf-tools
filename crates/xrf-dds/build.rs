/// Links the C++ runtime that `image_dds`'s encoder needs and does not link: `intel_tex_2` ships its ASTC helper as a
/// prebuilt C++ object, which names the runtime's exception personality. MSVC strips the unused helper; a Unix linker
/// that loads the object leaves the symbol undefined, as every Linux doc test linking this crate showed.
fn main() {
  println!("cargo:rerun-if-changed=build.rs");

  let os: String = std::env::var("CARGO_CFG_TARGET_OS").expect("Cargo sets the target OS");
  let family: String = std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default();

  match os.as_str() {
    "macos" | "ios" => println!("cargo:rustc-link-lib=dylib=c++"),
    _ if family == "unix" => println!("cargo:rustc-link-lib=dylib=stdc++"),
    _ => {}
  }
}
