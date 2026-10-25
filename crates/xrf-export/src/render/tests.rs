use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfError;
use xrf_test_utils::GeneratedResources;
use xrf_utils::LineEndings;
use xrf_xml::{XmlDocument, XmlParseOptions};

use crate::{
  ExternCallable, ExternDocumentation, ExternExport, ExternFormat, ExternManifest, ExternParameter, ExternValue,
};

fn callable_manifest() -> ExternManifest {
  ExternManifest {
    exports: BTreeMap::from([(
      String::from("test.run"),
      ExternExport::Callable(ExternCallable {
        doc: Some(ExternDocumentation {
          description: Some(String::from("Runs <it>.")),
          returns: Some(String::from("Whether it ran.")),
        }),
        params: vec![
          ExternParameter {
            doc: Some(String::from("Its id.")),
            name: String::from("id"),
            optional: None,
            type_name: String::from("TNumberId"),
          },
          ExternParameter {
            doc: None,
            name: String::from("count"),
            optional: Some(true),
            type_name: String::from("number"),
          },
        ],
        returns: String::from("boolean"),
        source: String::from("src/test.ts"),
      }),
    )]),
  }
}

fn value_manifest(type_name: &str) -> ExternManifest {
  ExternManifest {
    exports: BTreeMap::from([(
      String::from("test.value"),
      ExternExport::Value(ExternValue {
        doc: None,
        source: String::from("src/test.ts"),
        type_name: String::from(type_name),
      }),
    )]),
  }
}

fn create_test_root(name: &str) -> PathBuf {
  GeneratedResources::create_dir(&format!("extern-render/{name}")).unwrap()
}

#[test]
fn renders_stable_json_with_crlf_by_default() {
  let rendered: String = callable_manifest().render(ExternFormat::Json, None).unwrap();

  assert!(rendered.contains("\r\n"));
  assert!(rendered.ends_with("\r\n"));
  assert!(rendered.contains("\"exports\""));
}

#[test]
fn line_endings_can_be_overridden() {
  let rendered: String = ExternManifest::default()
    .render(ExternFormat::Json, Some(LineEndings::Lf))
    .unwrap();

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

  let rendered: String = manifest.render(ExternFormat::Xml, None).unwrap();
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

// The layout an XML artifact is checked against, element by element.
#[test]
fn lays_a_callable_out_as_xml() {
  assert_eq!(
    callable_manifest().render(ExternFormat::Xml, None).unwrap(),
    "<externs>\n  <exports>\n    <export name=\"test.run\">\n      <doc>\n        <description>Runs &lt;it>.\
     </description>\n        <returns>Whether it ran.</returns>\n      </doc>\n      <params>\n        <param name=\"id\" \
     type=\"TNumberId\">\n          <doc>Its id.</doc>\n        </param>\n        <param name=\"count\" type=\"number\" \
     optional=\"true\"></param>\n      </params>\n      <returns>boolean</returns>\n      <source>src/test.ts</source>\n    \
     </export>\n  </exports>\n</externs>\n"
  );
}

#[test]
fn groups_html_by_namespace_with_each_contract_and_its_documentation() {
  let rendered: String = callable_manifest().render(ExternFormat::Html, None).unwrap();

  assert!(rendered.contains("<summary>test</summary>"), "{rendered}");
  assert!(
    rendered.contains("<code>(id: TNumberId, count?: number) =&gt; boolean</code>"),
    "{rendered}"
  );
  assert!(
    rendered.contains("<td class=\"docs\">Runs &lt;it&gt;.\nReturns: Whether it ran.\nid: Its id.</td>"),
    "{rendered}"
  );
}

#[test]
fn infers_every_published_format_from_a_destination() {
  // A surface that offers three formats names the one it wants in the destination and nowhere else.
  for (name, expected) in [
    ("extern.json", ExternFormat::Json),
    ("extern.XML", ExternFormat::Xml),
    ("extern.html", ExternFormat::Html),
    ("extern.htm", ExternFormat::Html),
  ] {
    assert_eq!(
      ExternFormat::from_extension(Path::new(name)).unwrap(),
      expected,
      "{name}"
    );
  }

  assert!(ExternFormat::from_extension(Path::new("extern.txt")).is_err());
  assert!(ExternFormat::from_extension(Path::new("extern")).is_err());

  // A name merely ending in a format's spelling is not that format: the extension is what follows the final dot.
  assert!(ExternFormat::from_extension(Path::new("externjson")).is_err());

  // And a directory carrying a dot does not lend its extension to the file inside it.
  assert!(ExternFormat::from_extension(Path::new("out.json/extern")).is_err());
}

#[test]
fn publishes_the_rendered_artifact_over_whatever_was_there() {
  let root: PathBuf = create_test_root("write");
  // A directory the artifact has never been written into, which is what a save dialog can name.
  let path: PathBuf = root.join("nested/extern.json");

  ExternManifest::default()
    .write(&path, ExternFormat::Json, None)
    .unwrap();

  assert_eq!(
    fs::read_to_string(&path).unwrap(),
    ExternManifest::default().render(ExternFormat::Json, None).unwrap()
  );

  let manifest: ExternManifest = value_manifest("number");

  manifest.write(&path, ExternFormat::Json, None).unwrap();

  assert_eq!(
    fs::read_to_string(&path).unwrap(),
    manifest.render(ExternFormat::Json, None).unwrap()
  );
}

#[test]
fn reports_an_artifact_it_could_not_publish() {
  let root: PathBuf = create_test_root("refused");

  // The directory itself, which no rename can replace.
  let error = ExternManifest::default()
    .write(&root, ExternFormat::Json, None)
    .expect_err("a directory cannot be replaced by a file");

  assert!(
    error.to_string().contains("Failed to write the extern artifact"),
    "{error}"
  );
}

// A JSON artifact is compared as the manifest it reads back as, an XML or HTML one as text whatever its line endings.
#[test]
fn verifies_an_artifact_against_the_manifest_it_should_describe() {
  let root: PathBuf = create_test_root("verify");
  let manifest: ExternManifest = callable_manifest();

  for format in [ExternFormat::Json, ExternFormat::Xml, ExternFormat::Html] {
    let path: PathBuf = root.join(format!("extern.{}", format.as_str()));
    let other_endings: LineEndings = match format.default_line_endings() {
      LineEndings::Lf => LineEndings::Crlf,
      LineEndings::Crlf => LineEndings::Lf,
    };

    manifest.write(&path, format, Some(other_endings)).unwrap();
    manifest.verify_artifact(&path, format, None).unwrap();

    let error: XrfError = value_manifest("number")
      .verify_artifact(&path, format, None)
      .expect_err("another manifest");

    assert!(
      error.to_string().contains("does not match the parsed declarations"),
      "{error}"
    );
  }

  fs::write(root.join("broken.json"), "{").unwrap();

  assert!(matches!(
    manifest.verify_artifact(&root.join("broken.json"), ExternFormat::Json, None),
    Err(XrfError::Invalid { .. })
  ));
  assert!(matches!(
    manifest.verify_artifact(&root.join("absent.json"), ExternFormat::Json, None),
    Err(XrfError::Io { .. })
  ));
}
