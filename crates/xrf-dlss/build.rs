/// Built with the SDK, binds NGX's C API, compiles its inline helpers, links its library, and records where its runtime
/// library lies, which a build that ships none loads.
fn main() {
  println!("cargo:rerun-if-env-changed=DLSS_SDK");
  println!("cargo:rerun-if-env-changed=VULKAN_SDK");
  println!("cargo:rerun-if-env-changed=LIBCLANG_PATH");
  println!("cargo:rerun-if-changed=src/ngx/wrapper.h");

  #[cfg(feature = "sdk")]
  sdk::bind();
}

#[cfg(feature = "sdk")]
mod sdk {
  use std::env;
  use std::path::PathBuf;

  /// The SDK's directories for a target: its static libraries and its runtime library.
  struct Platform {
    libraries: &'static str,
    runtime: &'static str,
  }

  /// On a Windows host with no `LIBCLANG_PATH` set, points bindgen at the libclang `cargo make fetch-sdks` placed in
  /// `.sdks/libclang`; elsewhere bindgen finds the system's.
  fn use_fetched_libclang() {
    if cfg!(windows) && env::var_os("LIBCLANG_PATH").is_none() {
      let fetched: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.sdks/libclang");

      // SAFETY: the build script runs on one thread, and bindgen reads the variable after this as it loads libclang.
      unsafe { env::set_var("LIBCLANG_PATH", fetched) };
    }
  }

  pub fn bind() {
    let sdk: PathBuf = PathBuf::from(env::var("DLSS_SDK").expect("DLSS_SDK is set; run `cargo make fetch-sdks`"));
    let vulkan: PathBuf =
      PathBuf::from(env::var("VULKAN_SDK").expect("VULKAN_SDK is set; run `cargo make fetch-sdks`"));
    let out: PathBuf = PathBuf::from(env::var("OUT_DIR").expect("Cargo sets OUT_DIR"));
    let os: String = env::var("CARGO_CFG_TARGET_OS").expect("Cargo sets the target OS");
    let arch: String = env::var("CARGO_CFG_TARGET_ARCH").expect("Cargo sets the target architecture");
    let is_crt_static: bool =
      env::var("CARGO_CFG_TARGET_FEATURE").is_ok_and(|it| it.split(',').any(|f| f == "crt-static"));
    let platform: Platform = match (os.as_str(), arch.as_str()) {
      ("windows", "x86_64") => Platform {
        libraries: "lib/Windows_x86_64/x64",
        runtime: "lib/Windows_x86_64/rel",
      },
      ("linux", "x86_64") => Platform {
        libraries: "lib/Linux_x86_64",
        runtime: "lib/Linux_x86_64/rel",
      },
      _ => panic!("The DLSS SDK has no libraries for {os} on {arch}"),
    };
    let includes: [PathBuf; 2] = [sdk.join("include"), vulkan.join("include")];
    let wrapper: PathBuf = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Cargo sets the manifest's directory"))
      .join("src/ngx/wrapper.h");

    println!(
      "cargo:rustc-link-search=native={}",
      sdk.join(platform.libraries).display()
    );
    println!(
      "cargo:rustc-env=XRF_DLSS_RUNTIME={}",
      sdk.join(platform.runtime).display()
    );

    match os.as_str() {
      // The library built against the C runtime the target links: the DLL one unless it links it statically.
      "windows" if is_crt_static => println!("cargo:rustc-link-lib=static=nvsdk_ngx_s"),
      "windows" => println!("cargo:rustc-link-lib=static=nvsdk_ngx_d"),
      _ => {
        println!("cargo:rustc-link-lib=static=nvsdk_ngx");
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=dylib=dl");
      }
    }

    use_fetched_libclang();

    // Vulkan's own types come from ash, so only NGX's are generated; its static inline helpers are compiled into a
    // library of their own.
    bindgen::Builder::default()
      .header(wrapper.to_string_lossy())
      .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
      .wrap_static_fns(true)
      .wrap_static_fns_path(out.join("wrap_static_fns"))
      .clang_args(includes.iter().map(|include| format!("-I{}", include.display())))
      .allowlist_item(".*NGX.*")
      .blocklist_item("Vk.*")
      .blocklist_item("PFN_vk.*")
      .blocklist_item(".*Cuda.*")
      .blocklist_item(".*CUDA.*")
      .generate()
      .expect("NGX's headers bind")
      .write_to_file(out.join("bindings.rs"))
      .expect("NGX's bindings are written");

    cc::Build::new()
      .file(out.join("wrap_static_fns.c"))
      .includes(&includes)
      .compile("wrap_static_fns");
  }
}
