use std::path::PathBuf;
use std::time::{Duration, Instant};

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::shader::shader_watcher::ShaderWatcher;

#[test]
fn tells_once_a_shader_source_is_written() {
  let root: PathBuf = build_absolute_generated_test_resource_path("shader-watcher");

  std::fs::create_dir_all(&root).expect("the folder is made");

  let watcher: ShaderWatcher = ShaderWatcher::new(&root);

  assert!(!watcher.take_changed());

  std::fs::write(root.join("probe.wgsl"), "fn main() {}").expect("the shader is written");

  let deadline: Instant = Instant::now() + Duration::from_secs(5);
  let mut is_told: bool = false;

  while !is_told && Instant::now() < deadline {
    std::thread::sleep(Duration::from_millis(20));
    is_told = watcher.take_changed();
  }

  assert!(is_told);
}
