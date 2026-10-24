use std::env;

fn main() {
  // Tauri's Windows runtime imports from version 6 of the common controls, which only a manifest asking for it loads:
  // the application gets one from tauri-build, and this crate's test binary, the one thing it links, would fail to start.
  if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!(
      "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' \
       version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
    );
  }
}
