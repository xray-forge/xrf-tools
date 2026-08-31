fn main() {
  xrf_build_info::emit();

  // Gives the executable the application mark in Explorer, shortcuts and the console window. `compile` reports
  // `NotWindows` and does nothing when the target is not Windows, so it needs no cfg of its own.
  println!("cargo:rerun-if-changed=xrf-cli.rc");
  println!("cargo:rerun-if-changed=../xrf-app/icons/icon.ico");

  embed_resource::compile("xrf-cli.rc", embed_resource::NONE)
    .manifest_required()
    .expect("failed to embed the windows resource");
}
