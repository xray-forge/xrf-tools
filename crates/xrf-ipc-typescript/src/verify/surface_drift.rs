/// How a fresh generation differs from the committed mirrors.
pub struct SurfaceDrift {
  pub removed: Vec<String>,
  pub changed: Vec<String>,
  pub added: Vec<String>,
}

impl SurfaceDrift {
  /// Whether the committed mirrors still describe types the Rust sources produce.
  pub fn is_breaking(&self) -> bool {
    !self.removed.is_empty() || !self.changed.is_empty()
  }

  /// Failure text naming what to regenerate and why, listing additions only as context.
  pub fn describe(&self) -> String {
    let mut report: String = String::from(
      "Committed frontend bindings no longer match the Rust sources.\nRun `cargo make generate-typescript`.\n",
    );

    for (label, entries) in [
      ("Missing from a fresh generation", &self.removed),
      ("Shape changed", &self.changed),
    ] {
      if !entries.is_empty() {
        report.push_str(&format!("\n{label}:\n"));

        for entry in entries {
          report.push_str(&format!("  {entry}\n"));
        }
      }
    }

    if !self.added.is_empty() {
      report.push_str("\nAlso absent from the mirrors, which alone would not fail:\n");

      for entry in &self.added {
        report.push_str(&format!("  {entry}\n"));
      }
    }

    report
  }
}
