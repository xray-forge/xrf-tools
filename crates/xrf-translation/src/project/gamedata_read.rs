use xrf_error::XrfResult;
use xrf_extension::XrayExtension;
use xrf_utils::to_portable_path_string;
use xrf_vfs::{XrayAsset, XrayDirectoryListing, XrayLogicalPath, XrayRoots, XraySearch};

use crate::language::TranslationLanguage;
use crate::project::constants::{MAP_DESC_DIRECTORY, OPENXRAY_XML};
use crate::project::descriptor::{
  TranslationFile, TranslationFinding, TranslationProjectDescriptor, TranslationProjectMode, TranslationSource,
};
use crate::types::{TranslationEntry, TranslationVariant};
use crate::xml::encoding::{TranslationIdentity, decode};
use crate::xml::read::parse_string_table;

/// Read a text root whose subdirectories are languages.
///
/// # Errors
///
/// Returns an error when the roots cannot be mounted or the prefix is not a logical path. Individual
/// files never fail the read; they are reported as findings instead.
pub fn read_gamedata(roots: &XrayRoots, prefix: &str) -> XrfResult<TranslationProjectDescriptor> {
  read_gamedata_in(&roots.to_search_plan()?.open()?, roots, prefix)
}

/// Read a text root over roots somebody else mounted.
///
/// # Errors
///
/// Returns an error when the prefix is not a logical path.
pub fn read_gamedata_in(
  search: &XraySearch,
  roots: &XrayRoots,
  prefix: &str,
) -> XrfResult<TranslationProjectDescriptor> {
  // Not narrowed to the prefix, because this walks down from the text root by name rather than filtering a flat
  // listing: the language directories are the structure being read, not noise to exclude.
  let root: Option<XrayLogicalPath> = if prefix.is_empty() {
    None
  } else {
    Some(XrayLogicalPath::new(prefix)?)
  };

  let mut descriptor: TranslationProjectDescriptor = TranslationProjectDescriptor {
    mode: TranslationProjectMode::Gamedata,
    roots: roots.clone(),
    prefix: prefix.to_owned(),
    ..Default::default()
  };

  for language in discover_languages(search, root.as_ref(), &mut descriptor.findings)? {
    // Non-recursive, matching the engine's own `text<language>*.xml` mask.
    let listing: XrayDirectoryListing = search.list_children(child(root.as_ref(), &language)?.as_str())?;
    let tables: Vec<XrayAsset> = listing.files.into_iter().filter(is_string_table).collect();

    for asset in &tables {
      merge_file(search, asset, &language, &mut descriptor);
    }

    // Read off the first file rather than assumed from the code: these directories carry languages
    // the enum has no mapping for, and their own declaration is the only statement that exists.
    if let Some(encoding) = tables
      .first()
      .and_then(|asset| read_declared_encoding(search, asset, &language))
    {
      descriptor.encodings.insert(language.clone(), encoding);
    }

    descriptor.languages.push(language);
  }

  descriptor.files.sort_keys();
  descriptor.finalize_editable();

  Ok(descriptor)
}

/// Whether an asset in a language directory is a string table the engine would load.
fn is_string_table(asset: &XrayAsset) -> bool {
  asset.get_logical_path().has_extension(XrayExtension::Xml)
}

fn merge_file(search: &XraySearch, asset: &XrayAsset, language: &str, descriptor: &mut TranslationProjectDescriptor) {
  let logical_path: &str = asset.get_logical_path().as_str();
  let name: String = asset.get_logical_path().file_name().to_owned();

  let entries: Vec<(String, String)> = match search
    .read_asset(asset)
    .and_then(|data| parse_string_table(identity(asset, language), &data))
  {
    Ok(entries) => entries,
    Err(error) => {
      descriptor.findings.push(TranslationFinding::new(
        "translations.unreadable",
        Some(logical_path.to_owned()),
        format!("Could not read this file, so its strings are missing: {error}"),
      ));

      return;
    }
  };

  let file: &mut TranslationFile = descriptor.files.entry(name).or_default();

  file.sources.insert(
    language.to_owned(),
    TranslationSource::new(
      logical_path,
      asset.to_physical_path().as_deref().map(to_portable_path_string),
    ),
  );

  for (id, text) in entries {
    let entry: &mut TranslationEntry = file.entries.entry(id.clone()).or_default();

    // Last occurrence wins, because that is the one `CStringTable::Load` leaves in the table.
    if entry
      .insert(language.to_owned(), Some(TranslationVariant::String(text)))
      .is_some()
    {
      descriptor.findings.push(TranslationFinding::new(
        "translations.duplicate",
        Some(logical_path.to_owned()),
        format!("'{id}' appears more than once; the game uses the last one and the others are ignored"),
      ));
    }
  }
}

/// Every language directory the text root exposes, sorted.
///
/// A directory rather than a filename rule, because that is where gamedata carries the language. The
/// listing is of logical children, so a language split across a loose tree and an archive is one
/// language here rather than two.
fn discover_languages(
  search: &XraySearch,
  root: Option<&XrayLogicalPath>,
  findings: &mut Vec<TranslationFinding>,
) -> XrfResult<Vec<String>> {
  let mut languages: Vec<String> = Vec::new();

  for name in search
    .list_children(root.map_or("", XrayLogicalPath::as_str))?
    .directories
  {
    if name.eq_ignore_ascii_case(MAP_DESC_DIRECTORY) {
      continue;
    }

    let directory: XrayLogicalPath = child(root, &name)?;
    let files: Vec<XrayAsset> = search.list_children(directory.as_str())?.files;

    if files.is_empty() {
      continue;
    }

    if files.len() == 1
      && files[0]
        .get_logical_path()
        .file_name()
        .eq_ignore_ascii_case(OPENXRAY_XML)
    {
      continue;
    }

    if TranslationLanguage::from_str_single(&name).is_err() {
      findings.push(TranslationFinding::new(
        "translations.unknown-language",
        Some(directory.as_str().to_owned()),
        format!("'{name}' is a language the game loads but XRF does not build"),
      ));
    }

    languages.push(name);
  }

  languages.sort();

  Ok(languages)
}

/// One directory below the text root, composed as a logical path rather than by string surgery.
///
/// An absent root is what an empty prefix means: the language directories sit at the top level.
fn child(prefix: Option<&XrayLogicalPath>, name: &str) -> XrfResult<XrayLogicalPath> {
  match prefix {
    Some(prefix) => prefix.join(name),
    None => XrayLogicalPath::new(name),
  }
}

/// The code page one language's files declare, read off the first of them.
fn read_declared_encoding(search: &XraySearch, asset: &XrayAsset, language: &str) -> Option<String> {
  let data: Vec<u8> = search.read_asset(asset).ok()?;

  Some(
    decode(identity(asset, language), &data)
      .ok()?
      .encoding
      .name()
      .to_lowercase(),
  )
}

/// How a gamedata string table spells its language: in the directory holding it, never in its name.
fn identity<'a>(asset: &'a XrayAsset, language: &'a str) -> TranslationIdentity<'a> {
  TranslationIdentity {
    file_name: asset.get_logical_path().file_name(),
    directory_name: Some(language),
  }
}
