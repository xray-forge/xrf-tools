use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Lists every WGSL module under `shaders/` for the shader library to embed, each named by its path there without the
/// extension, so a module is added by its file alone.
fn main() {
  let manifest: PathBuf = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Cargo sets the manifest's directory"));
  let out: PathBuf = PathBuf::from(env::var("OUT_DIR").expect("Cargo sets OUT_DIR"));
  let root: PathBuf = manifest.join("shaders");
  let mut modules: Vec<String> = Vec::new();

  // A directory is watched whole: a module added, changed or removed lists them again.
  println!("cargo:rerun-if-changed=shaders");

  list_modules(&root, &root, &mut modules);
  modules.sort();

  let entries: String = modules
    .iter()
    .map(|name| {
      format!("  ({name:?}, include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/shaders/{name}.wgsl\"))),\n")
    })
    .collect();

  fs::write(out.join("shader_modules.rs"), format!("&[\n{entries}]\n")).expect("the shader list is written");
}

/// Adds the name of every `.wgsl` file under `directory` to `modules`, as its path from `root` joined by `/`.
fn list_modules(root: &Path, directory: &Path, modules: &mut Vec<String>) {
  let entries = fs::read_dir(directory).expect("the shaders' directories are readable");

  for entry in entries {
    let path: PathBuf = entry.expect("a shaders' directory entry is readable").path();

    if path.is_dir() {
      list_modules(root, &path, modules);
    } else if path.extension().is_some_and(|extension| extension == "wgsl") {
      let relative: PathBuf = path
        .strip_prefix(root)
        .expect("a module lies under the root")
        .with_extension("");
      let parts: Vec<String> = relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();

      modules.push(parts.join("/"));
    }
  }
}
