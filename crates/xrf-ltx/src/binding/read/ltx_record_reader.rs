use std::borrow::Cow;
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

use rustc_hash::FxBuildHasher;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, read_as_string_from_w1251_encoded};

use crate::binding::{LtxImportExport, LtxRecord};
use crate::dialect::LtxStandardLowering;
use crate::document::{LtxItem, LtxItemKind, LtxParser};
use crate::syntax::fold_section_name;

/// Reads a record file one section at a time as it is parsed, without building a config of it.
///
/// Sections arrive in file order once their last field is read, as the [`crate::Ltx`] the file lowers to lists them:
/// root fields first when there are any, declared sections even when empty. The file is refused as
/// [`crate::Ltx::read_from_str`] refuses it, and a refusal outranks the visit's errors wherever each sits. A record
/// file holds up to hundreds of thousands of sections, whose config costs several times the text in memory.
pub struct LtxRecordReader;

/// A record imported from the section named by its index.
struct LtxIndexedRecord<T> {
  index: usize,
  imported: T,
}

/// One file's reading under way: the section open, and what the rules and the visit have answered so far.
struct LtxRecordReading<'t, V> {
  record: LtxRecord<'t>,
  /// Whether a header opened the open section, rather than fields before any header.
  is_declared: bool,
  /// Every section handed over so far, lowercased, to refuse a repeat in any case.
  read: HashSet<Cow<'t, str>, FxBuildHasher>,
  includes: Vec<&'t str>,
  visit: V,
  /// The visit's first error, after which it is not asked again.
  rejection: Option<XrfError>,
}

impl LtxRecordReader {
  /// Reads the file at `path` as strict Windows-1251, handing each section to `visit`.
  ///
  /// # Errors
  ///
  /// Returns an error naming the file when it cannot be read or decoded, for what [`Self::read_str`] refuses, or the
  /// first error `visit` returns.
  pub fn read_path<P, V>(path: P, visit: V) -> XrfResult
  where
    P: AsRef<Path>,
    V: FnMut(&LtxRecord<'_>) -> XrfResult,
  {
    let path: &Path = path.as_ref();
    let read = || -> XrfResult { Self::read_str(&read_as_string_from_w1251_encoded(&mut File::open(path)?)?, visit) };

    read().map_err(|error| XrfError::new_read_error(format!("{} was not read: {error}", format_path(path))))
  }

  /// Reads `text`, handing each section to `visit` until it returns an error.
  ///
  /// # Errors
  ///
  /// Returns an error, in this precedence as [`crate::Ltx::read_from_str`] would, when the text does not parse, names a
  /// section or include twice, or holds a DLTX operation; otherwise the first error `visit` returns.
  pub fn read_str<V>(text: &str, visit: V) -> XrfResult
  where
    V: FnMut(&LtxRecord<'_>) -> XrfResult,
  {
    let mut parser: LtxParser<'_> = LtxParser::new(text.chars());
    let mut reading: LtxRecordReading<'_, V> = LtxRecordReading::new(visit);
    let mut refusal: Option<XrfError> = None;

    while let Some(item) = parser.parse_next_item()? {
      // After the first refusal the rest is only parsed, for a parse error that would outrank it.
      if refusal.is_none()
        && let Err(error) = reading.push(&item)
      {
        refusal = Some(error);
      }
    }

    if let Some(refusal) = refusal {
      return Err(refusal);
    }

    reading.finish()
  }

  /// Imports a record from every section of the file at `path`, in file order, as from the config it lowers to.
  ///
  /// # Errors
  ///
  /// Returns an error for what [`Self::read_path`] refuses, or the first error an import returns.
  pub fn import_all<T, P>(path: P) -> XrfResult<Vec<T>>
  where
    T: LtxImportExport,
    P: AsRef<Path>,
  {
    let mut imported: Vec<T> = Vec::new();

    Self::read_path(path, |record| {
      imported.push(T::import(record.name(), record)?);

      Ok(())
    })?;

    Ok(imported)
  }

  /// Imports sections `0` to `count - 1` of the file at `path`, as by index from the config it lowers to.
  ///
  /// Sections may come in any order; other names are ignored.
  ///
  /// # Errors
  ///
  /// Returns an error for what [`Self::read_path`] refuses, for the lowest missing index, or for the lowest index whose
  /// import fails.
  pub fn import_indexed<T, P>(path: P, count: usize) -> XrfResult<Vec<T>>
  where
    T: LtxImportExport,
    P: AsRef<Path>,
  {
    // Held as the file has them rather than in `count` slots: the count is the caller's to believe, and a header naming
    // more records than the file holds must answer with the first one missing, not reserve room for all of them.
    let mut read: Vec<LtxIndexedRecord<T>> = Vec::new();
    let mut failure: Option<(usize, XrfError)> = None;

    Self::read_path(&path, |record| {
      let Some(index) = Self::index_named(record.name()).filter(|&index| index < count) else {
        return Ok(());
      };

      // A lower index failing first answers for the file whatever this one holds.
      if failure.as_ref().is_none_or(|(failed, _)| index < *failed) {
        match T::import(record.name(), record) {
          Ok(imported) => read.push(LtxIndexedRecord { index, imported }),
          Err(error) => failure = Some((index, error)),
        }
      }

      Ok(())
    })?;

    // Already in order when the file was written in order, which a sort sees in one pass.
    read.sort_unstable_by_key(|record| record.index);

    let mut read = read.into_iter().peekable();
    let mut imported: Vec<T> = Vec::with_capacity(read.len());

    for index in 0..count {
      if let Some((_, error)) = failure.take_if(|(failed, _)| *failed == index) {
        return Err(error);
      }

      imported.push(match read.next_if(|record| record.index == index) {
        Some(record) => record.imported,
        None => {
          return Err(XrfError::new_not_found_error(format!(
            "{} counts {count} records but holds no section [{index}]",
            format_path(path.as_ref())
          )));
        }
      });
    }

    Ok(imported)
  }

  /// The index a section name spells as `index.to_string()` would, and no other spelling of it.
  fn index_named(name: &str) -> Option<usize> {
    let digits: &[u8] = name.as_bytes();
    let is_canonical: bool =
      !digits.is_empty() && digits.iter().all(u8::is_ascii_digit) && (digits.len() == 1 || digits[0] != b'0');

    if is_canonical { name.parse().ok() } else { None }
  }
}

impl<'t, V> LtxRecordReading<'t, V>
where
  V: FnMut(&LtxRecord<'_>) -> XrfResult,
{
  fn new(visit: V) -> Self {
    let mut record: LtxRecord<'_> = LtxRecord::default();

    record.open_root();

    Self {
      record,
      is_declared: false,
      read: HashSet::default(),
      includes: Vec::new(),
      visit,
      rejection: None,
    }
  }

  /// Takes in one statement under [`LtxStandardLowering`]'s rules.
  ///
  /// # Errors
  ///
  /// Returns an error when a section is declared twice, an include is named twice, or the statement is a DLTX
  /// operation.
  fn push(&mut self, item: &LtxItem<&'t str>) -> XrfResult {
    if item.is_patch_operation() {
      return Err(LtxStandardLowering::new_patch_operation_error(item));
    }

    match &item.kind {
      LtxItemKind::Comment { .. } => {}

      LtxItemKind::Include { path, .. } => {
        if self.includes.contains(path) {
          return Err(LtxStandardLowering::new_repeated_include_error(item, path));
        }

        self.includes.push(*path);
      }

      LtxItemKind::Section { name, .. } => {
        // Handed over first, so a section declared twice is caught by the second header.
        self.hand_over();

        if self.read.contains(&fold_section_name(name)) {
          return Err(LtxStandardLowering::new_repeated_section_error(item, name));
        }

        self.record.open(name);
        self.is_declared = true;
      }

      LtxItemKind::Key { name, value, .. } => self.record.set(name, value.unwrap_or_default()),
    }

    Ok(())
  }

  /// Hands over the last section, then returns the visit's first error.
  fn finish(mut self) -> XrfResult {
    self.hand_over();

    match self.rejection {
      Some(rejection) => Err(rejection),
      None => Ok(()),
    }
  }

  /// Hands the open section to the visit when a config would hold it: declared even when empty, the root only with
  /// fields.
  fn hand_over(&mut self) {
    if !self.is_declared && self.record.is_empty() {
      return;
    }

    self.read.insert(fold_section_name(self.record.name()));

    if self.rejection.is_none()
      && let Err(error) = (self.visit)(&self.record)
    {
      self.rejection = Some(error);
    }
  }
}

#[cfg(test)]
mod test {
  use std::sync::atomic::{AtomicUsize, Ordering};

  use xrf_error::{XrfError, XrfResult};

  use crate::binding::{LtxImportExport, LtxRead, LtxRecordReader, LtxSectionRead, LtxWrite};
  use crate::ltx::Ltx;

  type Listed = (String, Vec<(String, String)>);

  fn list_read(text: &str) -> XrfResult<Vec<Listed>> {
    let mut listed: Vec<Listed> = Vec::new();

    LtxRecordReader::read_str(text, |record| {
      listed.push((
        String::from(record.name()),
        record
          .fields()
          .map(|(key, value)| (String::from(key), String::from(value)))
          .collect(),
      ));

      Ok(())
    })?;

    Ok(listed)
  }

  fn list_lowered(text: &str) -> XrfResult<Vec<Listed>> {
    Ok(
      Ltx::read_from_str(text)?
        .iter()
        .map(|(name, section)| {
          (
            String::from(name),
            section
              .iter()
              .map(|(key, value)| (String::from(key), String::from(value)))
              .collect(),
          )
        })
        .collect(),
    )
  }

  // Every ordered three lines of a pool of each statement the parser reads or refuses and each standard-rule refusal,
  // under each line break.
  #[test]
  fn hands_over_what_lowering_the_text_holds() {
    const LINES: [&str; 28] = [
      "",
      "; note",
      "; @xrf-ltx skip-inheritance",
      "[a]",
      "  [b]:a, , c ; header note",
      "[a]x trailing caption",
      "[]",
      "k = v",
      "k =",
      "k = мова ; why",
      "j = 1",
      "bare",
      "#include \"x.ltx\"",
      "#include(\"y.LTX\") ; note",
      "![a]",
      "@[a]:b",
      "!![a]",
      "!k",
      ">k = v",
      "<k = v",
      "[unclosed",
      "[",
      "#include",
      "#include \"\"",
      "#include \"x.ini\"",
      "=",
      "k = w",
      "[c]",
    ];

    let mut compared: usize = 0;

    for first in LINES {
      for second in LINES {
        for third in LINES {
          for separator in ["\n", "\r\n", "\r"] {
            let contents: String = [first, second, third].join(separator);

            match (list_read(&contents), list_lowered(&contents)) {
              (Ok(read), Ok(lowered)) => assert_eq!(read, lowered, "{contents:?}"),
              (Err(read), Err(lowered)) => assert_eq!(format!("{read:?}"), format!("{lowered:?}"), "{contents:?}"),
              (read, lowered) => panic!("{contents:?} read as {read:?}, lowered as {lowered:?}"),
            }

            compared += 1;
          }
        }
      }
    }

    assert_eq!(compared, 28 * 28 * 28 * 3);
  }

  // A config was read whole before any import, so a refusal of the file outranks an import's error.
  #[test]
  fn a_refusal_below_a_failed_visit_is_the_one_reported() {
    let refused = |text: &str| -> String {
      LtxRecordReader::read_str(text, |_| Err(XrfError::new_parsing_error("visit refused")))
        .expect_err("the text to be refused")
        .to_string()
    };

    assert!(refused("[a]\nk = v\n").contains("visit refused"));
    assert!(refused("[a]\nk = v\n[a]\n").contains("Duplicate sections"));
    assert!(refused("[a]\nk = v\n[not_closed\n").contains("closing bracket"));
  }

  #[test]
  fn asks_the_visit_nothing_after_its_first_error() {
    let mut visited: Vec<String> = Vec::new();

    let result: XrfResult = LtxRecordReader::read_str("[a]\n[b]\n[c]\n", |record| {
      visited.push(String::from(record.name()));

      if record.name() == "b" {
        Err(XrfError::new_parsing_error("b refused"))
      } else {
        Ok(())
      }
    });

    assert!(result.is_err());
    assert_eq!(visited, vec!["a", "b"]);
  }

  #[derive(Debug, PartialEq)]
  struct Point {
    value: u32,
  }

  impl LtxImportExport for Point {
    fn import<L: LtxRead>(section_name: &str, ltx: &L) -> XrfResult<Self> {
      let section: &L::Section = ltx
        .section(section_name)
        .ok_or_else(|| XrfError::new_parsing_error(format!("Point section '{section_name}' should be defined")))?;

      Ok(Self {
        value: section.read_field("value")?,
      })
    }

    fn export(&self, _: &str, _: &mut impl LtxWrite) -> XrfResult {
      unreachable!("only read here")
    }
  }

  fn import_lowered(text: &str, count: usize) -> XrfResult<Vec<Point>> {
    let ltx: Ltx = Ltx::read_from_str(text)?;

    (0..count)
      .map(|index| Point::import(&index.to_string(), &ltx))
      .collect()
  }

  fn import_read(text: &str, count: usize) -> XrfResult<Vec<Point>> {
    // A file of its own per call, as tests run in parallel.
    static CALLS: AtomicUsize = AtomicUsize::new(0);

    let name: String = format!(
      "ltx_record_reader/indexed_{}.ltx",
      CALLS.fetch_add(1, Ordering::Relaxed)
    );
    let path = xrf_test_utils::GeneratedResources::write(&name, text.as_bytes())?;
    let imported: XrfResult<Vec<Point>> = LtxRecordReader::import_indexed(&path, count);

    std::fs::remove_file(path)?;

    imported
  }

  // A count comes from another file, so one far past the file must not reserve room for every record it names.
  #[test]
  fn answers_a_count_past_the_file_with_the_first_record_missing() {
    let error: String = import_read("[0]\nvalue = 1\n[1]\nvalue = 2\n", usize::MAX)
      .expect_err("record 2 missing")
      .to_string();

    assert!(error.contains("holds no section [2]"), "{error}");
  }

  #[test]
  fn imports_every_section_as_a_walk_of_the_config_did() -> XrfResult {
    for text in [
      "[0]\nvalue = 1\n[a]\nvalue = 2\n",
      "value = 0\n[a]\nvalue = 2\n",
      "[a]\nvalue = 1\n[empty]\n[b]\nvalue = 3\n",
      "[a]\nvalue = x\n[b]\nvalue = y\n",
      "[a]\nvalue = 1\n[a]\nvalue = 2\n",
      "",
    ] {
      let path = xrf_test_utils::GeneratedResources::write("ltx_record_reader/all.ltx", text.as_bytes())?;
      let read: XrfResult<Vec<Point>> = LtxRecordReader::import_all(&path);
      let lowered: XrfResult<Vec<Point>> =
        Ltx::read_from_str(text).and_then(|ltx| ltx.iter().map(|(name, _)| Point::import(name, &ltx)).collect());

      std::fs::remove_file(path)?;

      // The same error, prefixed with the file.
      match (read, lowered) {
        (Ok(read), Ok(lowered)) => assert_eq!(read, lowered, "{text:?}"),
        (Err(read), Err(lowered)) => assert!(
          read.to_string().ends_with(&lowered.to_string()),
          "{text:?}: {read} against {lowered}"
        ),
        (read, lowered) => panic!("{text:?} read as {read:?}, lowered as {lowered:?}"),
      }
    }

    Ok(())
  }

  #[test]
  fn imports_indexed_records_as_lookups_by_name_did() -> XrfResult {
    for (text, count) in [
      ("[0]\nvalue = 1\n[1]\nvalue = 2\n", 2),
      ("[1]\nvalue = 2\n[0]\nvalue = 1\n", 2),
      (
        "[0]\nvalue = 1\n[extra]\nvalue = x\n[01]\nvalue = x\n[+1]\nvalue = x\n[1]\nvalue = 2\n[2]\nvalue = 3\n",
        2,
      ),
      ("[0]\nvalue = 1\n[2]\nvalue = 3\n", 3),
      ("[0]\nvalue = x\n[2]\nvalue = 3\n", 3),
      ("[2]\nvalue = x\n[0]\nvalue = 1\n", 3),
      ("[3]\nvalue = x\n[1]\nvalue = x\n[0]\nvalue = 1\n[2]\nvalue = 3\n", 4),
      ("", 0),
      ("[0]\nvalue = 1\n[0]\nvalue = 2\n", 1),
    ] {
      let read: XrfResult<Vec<Point>> = import_read(text, count);
      let lowered: XrfResult<Vec<Point>> = import_lowered(text, count);

      // The reader reports a missing record itself; any other error is the same, prefixed with the file.
      match (read, lowered) {
        (Ok(read), Ok(lowered)) => assert_eq!(read, lowered, "{text:?}"),
        (Err(read), Err(lowered)) => assert!(
          read.to_string().contains("holds no section") || read.to_string().ends_with(&lowered.to_string()),
          "{text:?}: {read} against {lowered}"
        ),
        (read, lowered) => panic!("{text:?} read as {read:?}, lowered as {lowered:?}"),
      }
    }

    Ok(())
  }
}
