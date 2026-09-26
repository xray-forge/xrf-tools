use std::ops::Index;
use std::sync::Arc;

use crate::ltx::{PropertyIter, SectionData};

/// One resolved section: the parents its header declared, and its fields in written order.
///
/// Both are private, as [`crate::Ltx`]'s section map is, because a field's text is shared with every section that
/// inherited it - see [`Self::insert`]. Read through [`Self::get`], [`Self::iter`] and [`Self::inherits_section`];
/// write through [`Self::insert`], [`Self::remove`] and [`Self::inherit`].
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Section {
  pub(crate) inherited: Vec<String>,
  pub(crate) data: SectionData,
  /// The config whose header declared this section, once a dialect has said so.
  pub(crate) origin: Option<Arc<str>>,
}

impl Section {
  /// Create an instance.
  pub fn new() -> Self {
    Default::default()
  }

  /// Get the number of the properties.
  pub fn len(&self) -> usize {
    self.data.len()
  }

  /// Check if properties has 0 elements.
  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  /// The config that declared this section, where a dialect recorded one.
  pub fn get_origin(&self) -> Option<&str> {
    self.origin.as_deref()
  }

  /// Say which config declared this section.
  ///
  /// Public for the same reason [`crate::Ltx::set_source_paths`] is: a dialect lives in another crate and has to stamp
  /// what it resolved.
  pub fn set_origin(&mut self, origin: Arc<str>) {
    self.origin = Some(origin);
  }

  /// Get an iterator of the properties.
  pub fn iter(&self) -> PropertyIter<'_> {
    PropertyIter {
      inner: self.data.iter(),
    }
  }

  /// The fields with the handles they are stored under, for a caller that has to share them rather than read them.
  ///
  /// Separate from [`Self::iter`], which answers `&str` because that is what every reader wants; this exists so a
  /// record built beside a section can key on the same allocation instead of copying every key name.
  pub(crate) fn iter_shared(&self) -> impl Iterator<Item = (&Arc<str>, &Arc<str>)> {
    self.data.iter()
  }

  /// Give back the growth room this section's fields no longer need.
  pub(crate) fn shrink_to_fit(&mut self) {
    self.data.shrink_to_fit();
    self.inherited.shrink_to_fit();
  }

  /// Return true if property exist.
  pub fn contains_key(&self, key: &str) -> bool {
    self.data.contains_key(key)
  }

  /// Insert (key, value) pair by replace.
  ///
  /// Replacement is the only edit there is, and that is what keeps sharing safe: the text of a field may be held by
  /// every section that inherited it, so nothing hands out a mutable reference to it. An existing key keeps the
  /// position it already had.
  ///
  /// `AsRef<str>` rather than `Into<Arc<str>>`, which no `&String` satisfies, and rather than `Into<String>`, which
  /// would build a growable string only to share it. One allocation either way, sized to the value exactly.
  /// `Self::insert_shared` is the door for text this crate already holds.
  pub fn insert<K, V>(&mut self, key: K, value: V)
  where
    K: AsRef<str>,
    V: AsRef<str>,
  {
    self.data.insert(Arc::from(key.as_ref()), Arc::from(value.as_ref()));
  }

  /// Insert a field whose text is already held, sharing that allocation rather than making another.
  pub fn insert_shared(&mut self, key: Arc<str>, value: Arc<str>) {
    self.data.insert(key, value);
  }

  /// Copy another section's fields into this one, sharing their text rather than duplicating it.
  ///
  /// What inheritance is: a child holds its parents' fields, and the parent still holds them too. An existing key keeps
  /// the position it already had and takes the new value, which is the ordering repeated [`Self::insert`] gave.
  pub(crate) fn extend_shared(&mut self, section: &Self) {
    self.data.extend(
      section
        .data
        .iter()
        .map(|(key, value)| (Arc::clone(key), Arc::clone(value))),
    );
  }

  /// Return true if section inherits another section.
  pub fn inherits_section(&self, parent_section: &str) -> bool {
    self.inherited.iter().any(|inherited| inherited == parent_section)
  }

  /// Insert (key, value) pair by replace.
  pub fn inherit<S>(&mut self, parent_section: S)
  where
    S: Into<String>,
  {
    self.inherited.push(parent_section.into());
  }

  /// Merge another section into current one.
  pub fn merge(&mut self, section: Self) {
    self.data.extend(section.data);
  }

  /// Get the first value associate with the key.
  pub fn get(&self, key: &str) -> Option<&str> {
    self.data.get(key).map(|value| &**value)
  }

  /// Reads a value as `CInifile::r_bool` does: `on`, `yes`, `true` and `1`, in any case, are true, anything else false;
  /// `None` for a key the section lacks.
  pub fn get_bool(&self, key: &str) -> Option<bool> {
    self.get(key).map(|value| {
      let value: &str = value.trim();

      ["on", "yes", "true", "1"]
        .iter()
        .any(|it| value.eq_ignore_ascii_case(it))
    })
  }

  /// Reads a value as `CInifile::r_float` does, through `atof`: the number its text starts with, zero for text that
  /// starts with none; `None` for a key the section lacks.
  pub fn get_f32(&self, key: &str) -> Option<f32> {
    self.get(key).map(|value| parse_leading_float(value.trim_start()))
  }

  /// Remove the property with the first value of the key.
  pub fn remove(&mut self, key: &str) -> Option<Arc<str>> {
    self.data.shift_remove(key)
  }
}

impl<S: AsRef<str>> Index<S> for Section {
  type Output = str;

  fn index(&self, index: S) -> &str {
    let section: &str = index.as_ref();

    match self.get(section) {
      Some(property) => property,
      None => panic!("Key `{}` does not exist", section),
    }
  }
}

/// `atof`: the longest prefix of the text that reads as a decimal number, sign and exponent included, and zero where
/// none does.
fn parse_leading_float(text: &str) -> f32 {
  let bytes: &[u8] = text.as_bytes();
  let digits = |from: usize| -> usize { bytes[from..].iter().take_while(|it| it.is_ascii_digit()).count() };
  let mut end: usize = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
  let whole: usize = digits(end);

  end += whole;

  let fraction: usize = if bytes.get(end) == Some(&b'.') {
    digits(end + 1)
  } else {
    0
  };

  if bytes.get(end) == Some(&b'.') && whole + fraction > 0 {
    end += 1 + fraction;
  }

  if whole + fraction == 0 {
    return 0.0;
  }

  // An exponent counts only with a digit after it, as `strtod` takes it.
  if matches!(bytes.get(end), Some(b'e' | b'E')) {
    let sign: usize = usize::from(matches!(bytes.get(end + 1), Some(b'+' | b'-')));
    let exponent: usize = digits(end + 1 + sign);

    if exponent > 0 {
      end += 1 + sign + exponent;
    }
  }

  text[..end].parse().unwrap_or(0.0)
}

#[cfg(test)]
mod test {
  use crate::ltx::Section;

  #[test]
  fn property_replace() {
    let mut props: Section = Section::new();

    assert_eq!(props.len(), 0);

    props.insert("k1", "v1");

    assert_eq!(props.len(), 1);
    assert_eq!(props.get("k1"), Some("v1"));

    props.insert("k1", "v2");

    assert_eq!(props.len(), 1);
    assert_eq!(props.get("k1"), Some("v2"));
  }

  #[test]
  fn reads_a_flag_as_r_bool_does() {
    let mut section: Section = Section::new();

    section.insert("on", "On");
    section.insert("one", " 1 ");
    section.insert("no", "no");
    section.insert("longer", "truer");

    assert_eq!(section.get_bool("on"), Some(true));
    assert_eq!(section.get_bool("one"), Some(true));
    assert_eq!(section.get_bool("no"), Some(false));
    assert_eq!(section.get_bool("longer"), Some(false));
    assert_eq!(section.get_bool("missing"), None);
  }

  #[test]
  fn reads_a_number_as_atof_does() {
    let mut section: Section = Section::new();

    section.insert("plain", "2.5");
    section.insert("prefixed", " -0.25 ; a comment the reader kept");
    section.insert("exponent", "1e2x");
    section.insert("dangling", "3e");
    section.insert("dot", ".5");
    section.insert("none", "abc");

    assert_eq!(section.get_f32("plain"), Some(2.5));
    assert_eq!(section.get_f32("prefixed"), Some(-0.25));
    assert_eq!(section.get_f32("exponent"), Some(100.0));
    assert_eq!(section.get_f32("dangling"), Some(3.0));
    assert_eq!(section.get_f32("dot"), Some(0.5));
    assert_eq!(section.get_f32("none"), Some(0.0));
    assert_eq!(section.get_f32("missing"), None);
  }

  #[test]
  fn property_remove() {
    let mut props = Section::new();

    props.insert("k1", "v1");
    props.insert("k1", "v2");

    assert_eq!(props.remove("k1").as_deref(), Some("v2"));
    assert!(!props.contains_key("k1"));
  }
}
