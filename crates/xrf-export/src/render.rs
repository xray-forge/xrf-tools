use std::collections::BTreeMap;
use std::str::FromStr;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{LineEndings, apply_line_endings, format_path};
use xrf_xml::{escape_xml_attribute, escape_xml_text};

use crate::extern_manifest::{ExternCallable, ExternExport, ExternManifest, ExternParameter};

/// Output contract selected by `xrf-cli export-externs`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternFormat {
  Json,
  Xml,
  Html,
}

impl ExternFormat {
  /// Returns the repository default line ending for this output format.
  pub fn default_line_endings(self) -> LineEndings {
    match self {
      Self::Json => LineEndings::Crlf,
      Self::Xml | Self::Html => LineEndings::Lf,
    }
  }

  /// Infers a format from a `.json`, `.xml`, `.html`, or `.htm` extension.
  ///
  /// # Errors
  ///
  /// Returns an error when the path has no supported extension.
  pub fn from_extension(path: &std::path::Path) -> XrfResult<Self> {
    let extension: String = path
      .extension()
      .and_then(|extension| extension.to_str())
      .unwrap_or_default()
      .to_ascii_lowercase();

    match extension.as_str() {
      "json" => Ok(Self::Json),
      "xml" => Ok(Self::Xml),
      "html" | "htm" => Ok(Self::Html),
      _ => Err(XrfError::new_invalid_error(format!(
        "Cannot infer extern export format from '{}'; use --format.",
        format_path(path)
      ))),
    }
  }
}

impl FromStr for ExternFormat {
  type Err = XrfError;

  fn from_str(value: &str) -> XrfResult<Self> {
    match value {
      "json" => Ok(Self::Json),
      "xml" => Ok(Self::Xml),
      "html" => Ok(Self::Html),
      _ => Err(XrfError::new_invalid_error(format!(
        "Unsupported extern export format '{value}'. Expected json, xml, or html."
      ))),
    }
  }
}

/// Renders an extern manifest and terminates it with the selected line ending.
///
/// When `line_endings` is `None`, JSON uses CRLF and XML/HTML use LF.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn render_extern_manifest(
  manifest: &ExternManifest,
  format: ExternFormat,
  line_endings: Option<LineEndings>,
) -> XrfResult<String> {
  let content: String = match format {
    ExternFormat::Json => serde_json::to_string_pretty(manifest)?,
    ExternFormat::Xml => render_xml(manifest),
    ExternFormat::Html => render_html(manifest),
  };
  let ending: LineEndings = line_endings.unwrap_or_else(|| format.default_line_endings());

  Ok(apply_line_endings(&format!("{content}\n"), ending))
}

fn render_xml(manifest: &ExternManifest) -> String {
  let mut result: String = String::from("<externs>\n  <exports>");

  for (name, export) in &manifest.exports {
    result.push_str(&format!("\n    <export name=\"{}\">", escape_xml_attribute(name)));
    append_xml_export(&mut result, export, 6);
    result.push_str("\n    </export>");
  }

  result.push_str("\n  </exports>\n</externs>");
  result
}

fn append_xml_export(result: &mut String, export: &ExternExport, indentation: usize) {
  match export {
    ExternExport::Callable(value) => {
      append_xml_documentation(result, value.doc.as_ref(), indentation);
      result.push_str(&format!("\n{}<params>", " ".repeat(indentation)));
      for parameter in &value.params {
        result.push_str(&format!(
          "\n{}<param name=\"{}\" type=\"{}\"{}>",
          " ".repeat(indentation + 2),
          escape_xml_attribute(&parameter.name),
          escape_xml_attribute(&parameter.type_name),
          if parameter.optional.is_some_and(|value| value) {
            " optional=\"true\""
          } else {
            ""
          },
        ));
        if let Some(doc) = &parameter.doc {
          result.push_str(&format!(
            "\n{}<doc>{}</doc>",
            " ".repeat(indentation + 4),
            escape_xml_text(doc)
          ));
          result.push_str(&format!("\n{}</param>", " ".repeat(indentation + 2)));
        } else {
          result.push_str("</param>");
        }
      }
      result.push_str(&format!("\n{}</params>", " ".repeat(indentation)));
      result.push_str(&format!(
        "\n{}<returns>{}</returns>",
        " ".repeat(indentation),
        escape_xml_text(&value.returns)
      ));
      result.push_str(&format!(
        "\n{}<source>{}</source>",
        " ".repeat(indentation),
        escape_xml_text(&value.source)
      ));
    }
    ExternExport::Value(value) => {
      append_xml_documentation(result, value.doc.as_ref(), indentation);
      result.push_str(&format!(
        "\n{}<source>{}</source>",
        " ".repeat(indentation),
        escape_xml_text(&value.source)
      ));
      result.push_str(&format!(
        "\n{}<type>{}</type>",
        " ".repeat(indentation),
        escape_xml_text(&value.type_name)
      ));
    }
  }
}

fn append_xml_documentation(
  result: &mut String,
  documentation: Option<&crate::extern_manifest::ExternDocumentation>,
  indentation: usize,
) {
  let Some(documentation) = documentation else {
    return;
  };
  result.push_str(&format!("\n{}<doc>", " ".repeat(indentation)));
  if let Some(description) = &documentation.description {
    result.push_str(&format!(
      "\n{}<description>{}</description>",
      " ".repeat(indentation + 2),
      escape_xml_text(description)
    ));
  }
  if let Some(returns) = &documentation.returns {
    result.push_str(&format!(
      "\n{}<returns>{}</returns>",
      " ".repeat(indentation + 2),
      escape_xml_text(returns)
    ));
  }
  result.push_str(&format!("\n{}</doc>", " ".repeat(indentation)));
}

fn render_html(manifest: &ExternManifest) -> String {
  let groups: BTreeMap<String, Vec<(&String, &ExternExport)>> = manifest.exports.iter().fold(
    BTreeMap::new(),
    |mut groups: BTreeMap<String, Vec<(&String, &ExternExport)>>, entry| {
      let namespace: String = extern_namespace(entry.0, entry.1);
      groups.entry(namespace).or_default().push(entry);
      groups
    },
  );
  let mut result: String = String::from(
    "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <title>XRF extern reference</title>\n  <style>body{font:14px system-ui,sans-serif;margin:2rem;color:#202124}details{margin:1rem 0}summary{cursor:pointer;font-weight:600}table{border-collapse:collapse;width:100%;margin-top:.5rem}th,td{border:1px solid #dadce0;padding:.5rem;text-align:left;vertical-align:top}th{background:#f8f9fa}code{white-space:nowrap}.docs{white-space:pre-wrap}</style>\n</head>\n<body>\n  <h1>XRF extern reference</h1>",
  );

  for (namespace, entries) in groups {
    result.push_str(&format!("\n  <details>\n    <summary>{}</summary>\n    <table>\n      <thead><tr><th>Name</th><th>Contract</th><th>Documentation</th><th>Source</th></tr></thead>\n      <tbody>", escape_html(&namespace)));
    for (name, export) in entries {
      result.push_str(&format!("\n        <tr><td><code>{}</code></td><td><code>{}</code></td><td class=\"docs\">{}</td><td><code>{}</code></td></tr>", escape_html(name), escape_html(&extern_contract(export)), render_html_docs(export), escape_html(export.source())));
    }
    result.push_str("\n      </tbody>\n    </table>\n  </details>");
  }

  result.push_str("\n</body>\n</html>");
  result
}

fn extern_namespace(name: &str, export: &ExternExport) -> String {
  name
    .split_once('.')
    .map(|(namespace, _)| namespace.into())
    .unwrap_or_else(|| {
      export
        .source()
        .split('/')
        .next_back()
        .unwrap_or("globals")
        .trim_end_matches(".ts")
        .into()
    })
}

fn extern_contract(export: &ExternExport) -> String {
  match export {
    ExternExport::Callable(value) => format!(
      "({}) => {}",
      value
        .params
        .iter()
        .map(render_parameter_contract)
        .collect::<Vec<String>>()
        .join(", "),
      value.returns,
    ),
    ExternExport::Value(value) => value.type_name.clone(),
  }
}

fn render_parameter_contract(parameter: &ExternParameter) -> String {
  format!(
    "{}{}: {}",
    parameter.name,
    if parameter.optional.is_some_and(|value| value) {
      "?"
    } else {
      ""
    },
    parameter.type_name,
  )
}

fn render_html_docs(export: &ExternExport) -> String {
  let mut parts: Vec<String> = Vec::new();
  if let Some(documentation) = export.documentation() {
    if let Some(description) = &documentation.description {
      parts.push(escape_html(description));
    }
    if let Some(returns) = &documentation.returns {
      parts.push(format!("Returns: {}", escape_html(returns)));
    }
  }
  if let ExternExport::Callable(ExternCallable { params, .. }) = export {
    for parameter in params {
      if let Some(doc) = &parameter.doc {
        parts.push(format!("{}: {}", escape_html(&parameter.name), escape_html(doc)));
      }
    }
  }
  parts.join("\n")
}

fn escape_html(value: &str) -> String {
  value
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;

  use super::{ExternFormat, LineEndings, render_extern_manifest};
  use crate::{ExternCallable, ExternExport, ExternManifest, ExternParameter, ExternValue};
  use xrf_xml::{XmlDocument, XmlParseOptions};

  #[test]
  fn renders_stable_json_with_crlf_by_default() {
    let manifest: ExternManifest = ExternManifest {
      exports: BTreeMap::from([(
        String::from("test.run"),
        ExternExport::Callable(ExternCallable {
          doc: None,
          params: vec![ExternParameter {
            doc: None,
            name: String::from("id"),
            optional: None,
            type_name: String::from("TNumberId"),
          }],
          returns: String::from("void"),
          source: String::from("src/test.ts"),
        }),
      )]),
    };

    let rendered: String = render_extern_manifest(&manifest, ExternFormat::Json, None).unwrap();

    assert!(rendered.contains("\r\n"));
    assert!(rendered.ends_with("\r\n"));
    assert!(rendered.contains("\"exports\""));
  }

  #[test]
  fn line_endings_can_be_overridden() {
    let rendered: String =
      render_extern_manifest(&ExternManifest::default(), ExternFormat::Json, Some(LineEndings::Lf)).unwrap();

    assert!(!rendered.contains("\r\n"));
  }

  #[test]
  fn renders_well_formed_xml_with_escaped_values() {
    let manifest: ExternManifest = ExternManifest {
      exports: BTreeMap::from([(
        String::from("test.<run>&\"'"),
        ExternExport::Value(ExternValue {
          doc: None,
          source: String::from("src/<test>&\"'.ts"),
          type_name: String::from("Record<A, B> & C"),
        }),
      )]),
    };

    let rendered: String = render_extern_manifest(&manifest, ExternFormat::Xml, None).unwrap();
    let document: XmlDocument = XmlDocument::parse(&rendered, XmlParseOptions::default()).unwrap();
    let export = document.elements_named("export").next().unwrap();

    assert_eq!(export.attribute("name"), Some("test.<run>&\"'"));
    assert_eq!(
      document.elements_named("source").next().unwrap().text(),
      "src/<test>&\"'.ts"
    );
    assert_eq!(
      document.elements_named("type").next().unwrap().text(),
      "Record<A, B> & C"
    );
  }
}
