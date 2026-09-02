use std::collections::HashMap;
use std::fs;
use std::fs::ReadDir;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;
use xrf_xml::{XmlDocument, XmlElement, XmlParseOptions};

use crate::constants::{XML_ATTRIBUTE_ID, XML_ATTRIBUTE_NAME, XML_TAG_FILE, XML_TAG_TEXTURE, XML_TAG_WINDOW};
use crate::data::{TextureFileDescriptor, TextureSpriteDescriptor};
use crate::description::PackDescriptionOptions;

pub struct XmlDescriptionCollection {
  pub files: HashMap<String, TextureFileDescriptor>,
}

impl XmlDescriptionCollection {
  /// Narrow the described files down to the ones requested by name, in the order requested.
  ///
  /// A description usually names several sheets, and packing rewrites every one of them. Selecting by
  /// name keeps a change to a single sheet from touching its neighbours. An unknown name is an error
  /// rather than a silently empty run, because that is almost always a typo.
  pub fn select_files(&self, options: &PackDescriptionOptions) -> XrfResult<Vec<&TextureFileDescriptor>> {
    if options.files.is_empty() {
      return Ok(self.files.values().collect());
    }

    let mut selected: Vec<&TextureFileDescriptor> = Vec::new();
    let mut unknown: Vec<&str> = Vec::new();

    for name in &options.files {
      let matched: Vec<&TextureFileDescriptor> = self
        .files
        .values()
        .filter(|it| Self::is_file_named(&it.name, name))
        .collect();

      match matched.len() {
        1 => selected.push(matched[0]),
        0 => unknown.push(name),
        _ => {
          return Err(XrfError::new_texture_processing_error(format!(
            "Expected '{}' to name a single described file, it matches {}",
            name,
            matched
              .iter()
              .map(|it| it.name.as_str())
              .collect::<Vec<&str>>()
              .join(", ")
          )));
        }
      }
    }

    if !unknown.is_empty() {
      let mut available: Vec<&str> = self.files.values().map(|it| it.name.as_str()).collect();
      available.sort_unstable();

      return Err(XrfError::new_texture_processing_error(format!(
        "Expected requested files to be described in {}, not found: {}, available: {}",
        format_path(&options.description),
        unknown.join(", "),
        available.join(", ")
      )));
    }

    Ok(selected)
  }

  /// Whether a described file name refers to the requested one.
  ///
  /// Described names carry their directory, as in `ui\ui_actor_weapons`, which is awkward to type on a
  /// command line. The bare file name is accepted too, and the separator may be given either way.
  fn is_file_named(described: &str, requested: &str) -> bool {
    let normalize = |value: &str| value.replace('\\', "/").to_lowercase();

    let described: String = normalize(described);
    let requested: String = normalize(requested);

    described == requested || described.rsplit('/').next().is_some_and(|base| base == requested)
  }

  /// Get descriptions from provided options.
  /// Handle both directory and single file as inputs.
  pub fn get_descriptions(options: &PackDescriptionOptions) -> XrfResult<Self> {
    if options.description.is_dir() {
      xrf_output::info!(
        options.output,
        "Check texture descriptions from dir: {}",
        format_path(&options.description)
      );

      let mut files: HashMap<String, TextureFileDescriptor> = HashMap::new();
      let entries: ReadDir = fs::read_dir(&options.description)?;

      for entry in entries.flatten() {
        let path: PathBuf = entry.path();

        if let Some(extension) = path.extension()
          && extension == "xml"
        {
          let descriptions: HashMap<String, TextureFileDescriptor> = Self::get_description(options, &path)?;

          descriptions
            .into_iter()
            .for_each(|(name, description)| match files.get_mut(&name) {
              None => {
                files.insert(name, description);
              }
              Some(existing) => {
                xrf_output::verbose!(options.output, "Merging textures for {name}");

                existing.sprites.extend(description.sprites);
              }
            })
        }
      }

      Ok(Self { files })
    } else {
      Ok(Self {
        files: Self::get_description(options, &options.description)?,
      })
    }
  }

  /// Get descriptions from provided file path.
  pub fn get_description(
    options: &PackDescriptionOptions,
    path: &Path,
  ) -> XrfResult<HashMap<String, TextureFileDescriptor>> {
    xrf_output::verbose!(options.output, "Found texture description: {}", format_path(path));

    let mut descriptions: HashMap<String, TextureFileDescriptor> = HashMap::new();

    let contents: Vec<u8> = fs::read(path)?;
    let document: XmlDocument = match XmlDocument::parse_bytes(&contents, XmlParseOptions { allow_dtd: true }) {
      Ok(doc) => doc,
      Err(error) => {
        if options.is_strict {
          return Err(XrfError::new_parsing_error(format!(
            "Failed to parse xml: {} - {}",
            format_path(path),
            error
          )));
        }

        xrf_output::warning!(
          options.output,
          "Error parsing XML file: {} - {}",
          format_path(path),
          error
        );
        return Ok(HashMap::new());
      }
    };

    let window: Option<&XmlElement> = (document.root().name() == XML_TAG_WINDOW).then_some(document.root());

    if let Some(window) = window {
      for file in window.children_named(XML_TAG_FILE) {
        let file_name: Option<&str> = file.attribute(XML_ATTRIBUTE_NAME);

        if let Some(file_name) = file_name {
          xrf_output::verbose!(options.output, "Parsing file: {file_name}");

          let mut file_description: TextureFileDescriptor = TextureFileDescriptor::new(file_name);

          for node in file.descendants_named(XML_TAG_TEXTURE) {
            if let Some(sprite) = TextureSpriteDescriptor::new_optional_from_node(node) {
              file_description.add_sprite(sprite);
            } else {
              xrf_output::warning!(
                options.output,
                "Skip texture node: {} ({})",
                node.attribute(XML_ATTRIBUTE_ID).unwrap_or("unknown"),
                node
                  .attributes()
                  .map(|(name, value)| format!("{name}={value}"))
                  .collect::<Vec<String>>()
                  .join(","),
              );
            }
          }

          if file_description.sprites.is_empty() {
            xrf_output::warning!(
              options.output,
              "Skip definitions node \"{file_name}\" without textures (in {})",
              format_path(path)
            );
          } else {
            match descriptions.get_mut(&file_description.name) {
              None => {
                descriptions.insert(file_description.name.clone(), file_description);
              }
              Some(existing) => {
                xrf_output::verbose!(options.output, "Merging textures for {file_name}");

                file_description
                  .sprites
                  .into_iter()
                  .for_each(|it| existing.sprites.push(it));
              }
            }
          }
        } else {
          xrf_output::warning!(options.output, "Invalid file node supplied without name attribute");
        }
      }
    } else {
      xrf_output::warning!(
        options.output,
        "Got no 'w' tag for file '{}'",
        format_path(&options.description)
      );
    }

    Ok(descriptions)
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_dds::ImageFormat;

  use super::XmlDescriptionCollection;
  use crate::PackDescriptionOptions;

  #[test]
  fn returns_an_error_for_invalid_xml_in_strict_mode() {
    let path: PathBuf =
      std::env::temp_dir().join(format!("xrf-texture-invalid-description-{}.xml", std::process::id()));

    let options: PackDescriptionOptions = PackDescriptionOptions {
      job: Default::default(),
      description: path.clone(),
      base: PathBuf::new(),
      output: Default::default(),
      output_path: PathBuf::new(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files: Vec::new(),
      is_strict: true,
    };

    fs::write(&path, "<w>").unwrap();

    let result = XmlDescriptionCollection::get_description(&options, &path);

    fs::remove_file(&path).unwrap();

    assert!(result.is_err());
  }
}

#[cfg(test)]
mod select_files_tests {
  use std::collections::HashMap;
  use std::path::PathBuf;

  use xrf_dds::ImageFormat;

  use super::XmlDescriptionCollection;
  use crate::data::TextureFileDescriptor;
  use crate::description::PackDescriptionOptions;

  fn collection_of(names: &[&str]) -> XmlDescriptionCollection {
    let mut files: HashMap<String, TextureFileDescriptor> = HashMap::new();

    for name in names {
      files.insert(String::from(*name), TextureFileDescriptor::new(*name));
    }

    XmlDescriptionCollection { files }
  }

  fn options_for(files: &[&str]) -> PackDescriptionOptions {
    PackDescriptionOptions {
      job: Default::default(),
      description: PathBuf::from("ui_actor_upgrades.xml"),
      base: PathBuf::new(),
      output: Default::default(),
      output_path: PathBuf::new(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files: files.iter().map(|it| String::from(*it)).collect(),
      is_strict: false,
    }
  }

  #[test]
  fn selects_every_file_when_none_requested() {
    let collection: XmlDescriptionCollection = collection_of(&[r"ui\ui_actor_weapons", r"ui\ui_actor_armor"]);

    assert_eq!(
      collection
        .select_files(&options_for(&[]))
        .expect("expect all files to be selected")
        .len(),
      2
    );
  }

  #[test]
  fn selects_by_bare_file_name() {
    let collection: XmlDescriptionCollection = collection_of(&[r"ui\ui_actor_weapons", r"ui\ui_actor_armor"]);

    let selected = collection
      .select_files(&options_for(&["ui_actor_weapons"]))
      .expect("expect the bare name to resolve");

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].name, r"ui\ui_actor_weapons");
  }

  #[test]
  fn selects_by_full_name_with_either_separator() {
    let collection: XmlDescriptionCollection = collection_of(&[r"ui\ui_actor_weapons"]);

    for requested in [r"ui\ui_actor_weapons", "ui/ui_actor_weapons"] {
      assert_eq!(
        collection
          .select_files(&options_for(&[requested]))
          .expect("expect either separator to resolve")
          .len(),
        1,
        "Expect {requested} to resolve"
      );
    }
  }

  #[test]
  fn rejects_an_unknown_name() {
    let collection: XmlDescriptionCollection = collection_of(&[r"ui\ui_actor_weapons"]);

    assert!(
      collection.select_files(&options_for(&["ui_typo"])).is_err(),
      "Expect an unknown name to fail rather than pack nothing"
    );
  }

  #[test]
  fn rejects_an_ambiguous_bare_name() {
    let collection: XmlDescriptionCollection = collection_of(&[r"ui\ui_actor_weapons", r"hud\ui_actor_weapons"]);

    assert!(
      collection.select_files(&options_for(&["ui_actor_weapons"])).is_err(),
      "Expect a bare name matching two described files to fail"
    );
  }
}
