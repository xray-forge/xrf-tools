use std::collections::BTreeMap;

use crate::manifest::{ExternExport, ExternManifest};

/// Lays a manifest out as a reference page, one collapsible table per namespace.
pub(crate) struct ExternHtmlRenderer;

impl ExternHtmlRenderer {
  const HEAD: &'static str = "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <title>XRF extern reference</title>\n  <style>body{font:14px system-ui,sans-serif;margin:2rem;color:#202124}details{margin:1rem 0}summary{cursor:pointer;font-weight:600}table{border-collapse:collapse;width:100%;margin-top:.5rem}th,td{border:1px solid #dadce0;padding:.5rem;text-align:left;vertical-align:top}th{background:#f8f9fa}code{white-space:nowrap}.docs{white-space:pre-wrap}</style>\n</head>\n<body>\n  <h1>XRF extern reference</h1>";

  /// The manifest as a page, without a final newline.
  pub fn render(manifest: &ExternManifest) -> String {
    let mut groups: BTreeMap<String, Vec<(&String, &ExternExport)>> = BTreeMap::new();

    for (name, export) in &manifest.exports {
      groups
        .entry(Self::get_namespace(name, export))
        .or_default()
        .push((name, export));
    }

    let mut result: String = String::from(Self::HEAD);

    for (namespace, entries) in groups {
      result.push_str(&format!(
        "\n  <details>\n    <summary>{}</summary>\n    <table>\n      <thead><tr><th>Name</th><th>Contract</th>\
         <th>Documentation</th><th>Source</th></tr></thead>\n      <tbody>",
        Self::escape(&namespace)
      ));

      for (name, export) in entries {
        result.push_str(&format!(
          "\n        <tr><td><code>{}</code></td><td><code>{}</code></td><td class=\"docs\">{}</td>\
           <td><code>{}</code></td></tr>",
          Self::escape(name),
          Self::escape(&export.get_contract()),
          Self::render_documentation(export),
          Self::escape(export.source())
        ));
      }

      result.push_str("\n      </tbody>\n    </table>\n  </details>");
    }

    result.push_str("\n</body>\n</html>");
    result
  }

  /// The first segment of a dotted name, or for a name of one segment, the file that declares it.
  fn get_namespace(name: &str, export: &ExternExport) -> String {
    match name.split_once('.') {
      Some((namespace, _)) => namespace.to_owned(),
      None => {
        let source: &str = export.source();

        source
          .rsplit_once('/')
          .map_or(source, |(_, file)| file)
          .trim_end_matches(".ts")
          .to_owned()
      }
    }
  }

  /// The description, the return, and each documented parameter, one to a line.
  fn render_documentation(export: &ExternExport) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(documentation) = export.documentation() {
      if let Some(description) = &documentation.description {
        parts.push(Self::escape(description));
      }

      if let Some(returns) = &documentation.returns {
        parts.push(format!("Returns: {}", Self::escape(returns)));
      }
    }

    if let ExternExport::Callable(callable) = export {
      for parameter in &callable.params {
        if let Some(doc) = &parameter.doc {
          parts.push(format!("{}: {}", Self::escape(&parameter.name), Self::escape(doc)));
        }
      }
    }

    parts.join("\n")
  }

  fn escape(value: &str) -> String {
    value
      .replace('&', "&amp;")
      .replace('<', "&lt;")
      .replace('>', "&gt;")
      .replace('"', "&quot;")
      .replace('\'', "&#39;")
  }
}
