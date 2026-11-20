use std::collections::HashMap;
use std::fs;
use std::fs::ReadDir;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_utils::format_path;
use xrf_xml::{XmlDocument, XmlElement, XmlParseOptions};

use crate::description::{
  DescriptionSpriteOptions, TextureFileDescriptor, TextureSpriteDescriptor, XML_ATTRIBUTE_ID, XML_ATTRIBUTE_NAME,
  XML_TAG_FILE, XML_TAG_TEXTURE, XML_TAG_WINDOW,
};
use crate::report::SpriteNotice;

pub(crate) struct XmlDescriptionCollection {
  pub files: HashMap<String, TextureFileDescriptor>,
}

impl XmlDescriptionCollection {
  /// Narrow the described files down to the ones requested by name, in the order requested.
  ///
  /// A description usually names several sheets, and packing rewrites every one of them. Selecting by
  /// name keeps a change to a single sheet from touching its neighbours. An unknown name is an error
  /// rather than a silently empty run, because that is almost always a typo.
  pub(crate) fn select_files(&self, options: &DescriptionSpriteOptions) -> XrfResult<Vec<&TextureFileDescriptor>> {
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

  /// Reads the descriptions `options` names, one file or every XML file in a directory, merging sheets described
  /// in several files and handing whatever is passed over to `on_notice`.
  pub(crate) fn get_descriptions(
    options: &DescriptionSpriteOptions,
    on_notice: &mut impl FnMut(SpriteNotice),
  ) -> XrfResult<Self> {
    if options.description.is_dir() {
      let mut files: HashMap<String, TextureFileDescriptor> = HashMap::new();
      let entries: ReadDir = fs::read_dir(&options.description)?;

      for entry in entries.flatten() {
        let path: PathBuf = entry.path();

        if XrayExtension::Xml.matches_path(&path) {
          let descriptions: HashMap<String, TextureFileDescriptor> = Self::get_description(options, &path, on_notice)?;

          descriptions
            .into_iter()
            .for_each(|(name, description)| match files.get_mut(&name) {
              None => {
                files.insert(name, description);
              }
              Some(existing) => existing.sprites.extend(description.sprites),
            });
        }
      }

      Ok(Self { files })
    } else {
      Ok(Self {
        files: Self::get_description(options, &options.description, on_notice)?,
      })
    }
  }

  /// Reads the sheets one description file declares, handing whatever is passed over to `on_notice`.
  pub(crate) fn get_description(
    options: &DescriptionSpriteOptions,
    path: &Path,
    on_notice: &mut impl FnMut(SpriteNotice),
  ) -> XrfResult<HashMap<String, TextureFileDescriptor>> {
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

        on_notice(SpriteNotice::DescriptionUnparsable {
          path: path.to_path_buf(),
          reason: error.to_string(),
        });

        return Ok(HashMap::new());
      }
    };

    let window: Option<&XmlElement> = (document.root().name() == XML_TAG_WINDOW).then_some(document.root());

    if let Some(window) = window {
      for file in window.children_named(XML_TAG_FILE) {
        let file_name: Option<&str> = file.attribute(XML_ATTRIBUTE_NAME);

        if let Some(file_name) = file_name {
          let mut file_description: TextureFileDescriptor = TextureFileDescriptor::new(file_name);

          for node in file.descendants_named(XML_TAG_TEXTURE) {
            if let Some(sprite) = TextureSpriteDescriptor::new_optional_from_node(node) {
              file_description.add_sprite(sprite);
            } else {
              on_notice(SpriteNotice::TextureNodeIncomplete {
                id: node.attribute(XML_ATTRIBUTE_ID).map(String::from),
                attributes: node
                  .attributes()
                  .map(|(name, value)| (String::from(name), String::from(value)))
                  .collect(),
              });
            }
          }

          if file_description.sprites.is_empty() {
            on_notice(SpriteNotice::FileWithoutTextures {
              name: String::from(file_name),
              path: path.to_path_buf(),
            });
          } else {
            match descriptions.get_mut(&file_description.name) {
              None => {
                descriptions.insert(file_description.name.clone(), file_description);
              }
              Some(existing) => existing.sprites.extend(file_description.sprites),
            }
          }
        } else {
          on_notice(SpriteNotice::FileWithoutName);
        }
      }
    } else {
      on_notice(SpriteNotice::DescriptionWithoutWindow {
        path: path.to_path_buf(),
      });
    }

    Ok(descriptions)
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_dds::ImageFormat;
  use xrf_test_utils::GeneratedResources;

  use super::XmlDescriptionCollection;
  use crate::DescriptionSpriteOptions;

  #[test]
  fn returns_an_error_for_invalid_xml_in_strict_mode() {
    let path: PathBuf =
      GeneratedResources::write("xml_description/invalid.xml", "<w>").expect("generated description to be written");

    let options: DescriptionSpriteOptions = DescriptionSpriteOptions {
      job: Default::default(),
      description: path.clone(),
      base: PathBuf::new(),
      output_path: PathBuf::new(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files: Vec::new(),
      is_strict: true,
      link_policy: Default::default(),
      is_forced: false,
    };

    let result = XmlDescriptionCollection::get_description(&options, &path, &mut |_| {});

    fs::remove_file(&path).unwrap();

    assert!(result.is_err());
  }

  /// A directory of descriptions was selected with a byte-exact `Path::extension` comparison, so a file the author
  /// named `ui_icon_equipment.XML` contributed nothing and the pack produced a sprite sheet missing every icon it
  /// declared, with no error to say why.
  #[test]
  fn reads_a_description_whose_name_was_typed_in_upper_case() {
    let directory: PathBuf = GeneratedResources::write(
      "xml_description/upper-case/UI_ICON_EQUIPMENT.XML",
      "<w><file name=\"ui_icon_equipment\"><texture id=\"wpn_ak74\" x=\"0\" y=\"0\" width=\"4\" height=\"4\">ui_icon_equipment</texture></file></w>",
    )
    .expect("generated description to be written")
    .parent()
    .expect("the description sits in a directory")
    .to_path_buf();

    let options: DescriptionSpriteOptions = DescriptionSpriteOptions {
      job: Default::default(),
      description: directory,
      base: PathBuf::new(),
      output_path: PathBuf::new(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files: Vec::new(),
      is_strict: true,
      link_policy: Default::default(),
      is_forced: false,
    };

    let collection: XmlDescriptionCollection =
      XmlDescriptionCollection::get_descriptions(&options, &mut |_| {}).expect("the directory to be read");

    assert_eq!(collection.files.len(), 1, "the upper-case description has to be read");
  }
}

#[cfg(test)]
mod select_files_tests {
  use std::collections::HashMap;
  use std::path::PathBuf;

  use xrf_dds::ImageFormat;

  use super::XmlDescriptionCollection;
  use crate::description::{DescriptionSpriteOptions, TextureFileDescriptor};

  fn collection_of(names: &[&str]) -> XmlDescriptionCollection {
    let mut files: HashMap<String, TextureFileDescriptor> = HashMap::new();

    for name in names {
      files.insert(String::from(*name), TextureFileDescriptor::new(*name));
    }

    XmlDescriptionCollection { files }
  }

  fn options_for(files: &[&str]) -> DescriptionSpriteOptions {
    DescriptionSpriteOptions {
      job: Default::default(),
      description: PathBuf::from("ui_actor_upgrades.xml"),
      base: PathBuf::new(),
      output_path: PathBuf::new(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files: files.iter().map(|it| String::from(*it)).collect(),
      is_strict: false,
      link_policy: Default::default(),
      is_forced: false,
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
