use serde::Serialize;
use xrf_app_core::TauriResult;
use xrf_utils::error_to_string;
use xrf_vfs::{XrayAsset, XrayAssetRules, XrayAssetType, XrayLogicalPath, XraySearch};

use crate::edit::{
  TextureDescriptorForm, TextureReferenceField, TextureReferenceRename, TextureSaveAsBump, TextureSaveAsFile,
  TextureSaveAsRole, TextureSaveAsTarget,
};

/// What a Save as writes: the texture's files under the copy's name, and the copy's descriptor fields that change to
/// name the copy's files.
///
/// Both the dialog and the save are built from this, so the person agrees to exactly what is written.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TextureSaveAsPlan {
  /// The copy's texture name, normalized.
  pub destination: String,
  /// Whether the original has bump map files of its own, which is when the bump choice matters.
  pub has_bump_pair: bool,
  /// Every file written, in write order: the descriptor always, the bump map files only when they are copied.
  pub files: Vec<TextureSaveAsFile>,
  /// The copy's fields that change from the original's file names to the copy's; none when the bump map is shared.
  pub renames: Vec<TextureReferenceRename>,
}

impl TextureSaveAsPlan {
  /// Plans a copy of the texture `target` names under its destination name, reading the roots through `search`.
  ///
  /// # Errors
  ///
  /// Returns an error when either name is not a valid texture name, the copy would take the original's name, no
  /// texture has the original's name, or the texture is served out of an archive.
  pub(crate) fn build(
    search: &XraySearch,
    target: &TextureSaveAsTarget,
    form: &TextureDescriptorForm,
    bump: TextureSaveAsBump,
  ) -> TauriResult<Self> {
    let reference: String = Self::normalize(&target.reference)?;
    let destination: String = Self::normalize(&target.destination)?;

    if destination == reference {
      return Err(format!(
        "'{reference}' is the original's name; choose another name for the copy, or use Save"
      ));
    }

    let texture: XrayAsset = Self::find(search, TextureSaveAsRole::Texture, &reference)?
      .ok_or_else(|| format!("No texture named '{reference}' was found in the open roots"))?;

    if texture.to_physical_path().is_none() {
      return Err(format!(
        "'{reference}' is served out of an archive, so there is no file to copy; extract it first"
      ));
    }

    Ok(Self {
      has_bump_pair: Self::find(search, TextureSaveAsRole::Bump, &reference)?.is_some(),
      files: Self::plan_files(search, &reference, &destination, bump)?,
      renames: match bump {
        TextureSaveAsBump::Own => Self::plan_renames(form, &reference, &destination),
        TextureSaveAsBump::Shared => Vec::new(),
      },
      destination,
    })
  }

  /// The texture's files that exist, plus its descriptor whether or not it exists; the bump map files only when owned.
  fn plan_files(
    search: &XraySearch,
    reference: &str,
    destination: &str,
    bump: TextureSaveAsBump,
  ) -> TauriResult<Vec<TextureSaveAsFile>> {
    let mut files: Vec<TextureSaveAsFile> = Vec::with_capacity(TextureSaveAsRole::ALL.len());

    for role in TextureSaveAsRole::ALL {
      if role.is_bump() && bump == TextureSaveAsBump::Shared {
        continue;
      }

      let from: Option<XrayAsset> = Self::find(search, role, reference)?;

      // A missing descriptor is written new from the form; a missing bump file is not invented.
      if from.is_none() && role != TextureSaveAsRole::Descriptor {
        continue;
      }

      let to: String = Self::logical_path(role, destination)?;

      files.push(TextureSaveAsFile {
        role,
        from: from.map(|asset| asset.get_logical_path().as_str().to_owned()),
        is_replacing: search.find(&to).map_err(error_to_string)?.is_some(),
        to,
      });
    }

    Ok(files)
  }

  /// The form's fields that name the original texture or its bump pair.
  fn plan_renames(form: &TextureDescriptorForm, reference: &str, destination: &str) -> Vec<TextureReferenceRename> {
    let owned: [(String, String); 3] = [
      TextureSaveAsRole::Texture,
      TextureSaveAsRole::Bump,
      TextureSaveAsRole::Companion,
    ]
    .map(|role| (role.name_for(reference), role.name_for(destination)));

    [
      (TextureReferenceField::Bump, &form.bump_name),
      (TextureReferenceField::Detail, &form.detail_name),
    ]
    .into_iter()
    .filter_map(|(field, value)| {
      // An empty or invalid name cannot name one of the texture's files.
      let named: String = XrayLogicalPath::normalize(value).ok()?;
      let (_, to) = owned.iter().find(|(from, _)| *from == named)?;

      Some(TextureReferenceRename {
        field,
        from: value.clone(),
        to: to.clone(),
      })
    })
    .collect()
  }

  /// A texture name lowercased with backslashes, and any `.dds` removed, as the engine reads it.
  fn normalize(reference: &str) -> TauriResult<String> {
    let rules: XrayAssetRules = Self::rules_of(XrayAssetType::Dds)?;
    let path: XrayLogicalPath =
      Self::join(rules, reference).map_err(|error| format!("'{reference}' is not a valid texture name: {error}"))?;

    rules
      .to_reference(&path)
      .ok_or_else(|| format!("'{reference}' is not a valid texture name"))
  }

  /// The file `role` names for the texture named `texture`, where the roots have one.
  fn find(search: &XraySearch, role: TextureSaveAsRole, texture: &str) -> TauriResult<Option<XrayAsset>> {
    search
      .find(&Self::logical_path(role, texture)?)
      .map_err(error_to_string)
  }

  /// The logical path of the file `role` names for the texture named `texture`.
  fn logical_path(role: TextureSaveAsRole, texture: &str) -> TauriResult<String> {
    Ok(
      Self::join(Self::rules_of(role.get_asset_type())?, &role.name_for(texture))
        .map_err(error_to_string)?
        .as_str()
        .to_owned(),
    )
  }

  /// A name joined below its kind's directory, such as `textures`.
  fn join(rules: XrayAssetRules, reference: &str) -> xrf_error::XrfResult<XrayLogicalPath> {
    XrayLogicalPath::new(rules.directory)?.join(&rules.to_logical_path(reference))
  }

  fn rules_of(asset_type: XrayAssetType) -> TauriResult<XrayAssetRules> {
    asset_type
      .get_rules()
      .ok_or_else(|| format!("{asset_type:?} files have no directory in the roots"))
  }
}
