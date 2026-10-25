use xrf_xml::{escape_xml_attribute, escape_xml_text};

use crate::manifest::{ExternDocumentation, ExternExport, ExternManifest};

/// Lays a manifest out as `<externs><exports>`, one `<export>` element per name.
pub(crate) struct ExternXmlRenderer;

impl ExternXmlRenderer {
  /// Spaces an element is indented by for each level it nests.
  const INDENT: usize = 2;

  /// The manifest as XML, without a final newline.
  pub fn render(manifest: &ExternManifest) -> String {
    let mut result: String = String::from("<externs>\n  <exports>");

    for (name, export) in &manifest.exports {
      result.push_str(&format!("\n    <export name=\"{}\">", escape_xml_attribute(name)));
      Self::append_export(&mut result, export, 3);
      result.push_str("\n    </export>");
    }

    result.push_str("\n  </exports>\n</externs>");
    result
  }

  fn append_export(result: &mut String, export: &ExternExport, depth: usize) {
    Self::append_documentation(result, export.documentation(), depth);

    match export {
      ExternExport::Callable(callable) => {
        Self::append_line(result, depth, "<params>");

        for parameter in &callable.params {
          let opening: String = format!(
            "<param name=\"{}\" type=\"{}\"{}>",
            escape_xml_attribute(&parameter.name),
            escape_xml_attribute(&parameter.type_name),
            if parameter.is_optional() {
              " optional=\"true\""
            } else {
              ""
            },
          );

          match &parameter.doc {
            Some(doc) => {
              Self::append_line(result, depth + 1, &opening);
              Self::append_line(result, depth + 2, &format!("<doc>{}</doc>", escape_xml_text(doc)));
              Self::append_line(result, depth + 1, "</param>");
            }
            None => Self::append_line(result, depth + 1, &format!("{opening}</param>")),
          }
        }

        Self::append_line(result, depth, "</params>");
        Self::append_text(result, depth, "returns", &callable.returns);
        Self::append_text(result, depth, "source", &callable.source);
      }
      ExternExport::Value(value) => {
        Self::append_text(result, depth, "source", &value.source);
        Self::append_text(result, depth, "type", &value.type_name);
      }
    }
  }

  fn append_documentation(result: &mut String, documentation: Option<&ExternDocumentation>, depth: usize) {
    let Some(documentation) = documentation else {
      return;
    };

    Self::append_line(result, depth, "<doc>");

    if let Some(description) = &documentation.description {
      Self::append_text(result, depth + 1, "description", description);
    }

    if let Some(returns) = &documentation.returns {
      Self::append_text(result, depth + 1, "returns", returns);
    }

    Self::append_line(result, depth, "</doc>");
  }

  /// One element holding escaped text, on a line of its own.
  fn append_text(result: &mut String, depth: usize, element: &str, text: &str) {
    Self::append_line(
      result,
      depth,
      &format!("<{element}>{}</{element}>", escape_xml_text(text)),
    );
  }

  fn append_line(result: &mut String, depth: usize, line: &str) {
    result.push('\n');
    result.push_str(&" ".repeat(depth * Self::INDENT));
    result.push_str(line);
  }
}
