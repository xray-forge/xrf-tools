use std::fmt::Display;
use std::mem;
use std::sync::Arc;

use crate::binding::LtxSectionWrite;
use crate::ltx::section::section_value_text::SectionValueText;
use crate::ltx::{Ltx, Section};

/// Writes fields into one named section of a config, creating the section on the first write.
///
/// A config can be written record by record, a setter each, so its costs are paid per field across hundreds of
/// thousands of records: the section is found once, not per field, a key the section before it already holds shares
/// that text instead of storing the name again, and a value is formatted straight into the text the field keeps. A
/// caller that only writes the file uses [`crate::LtxRecordWriter`] instead, which builds no config.
pub struct SectionSetter<'a> {
  ltx: &'a mut Ltx,
  /// The section's name until the first write places it, which then moves it into the config.
  section_name: String,
  /// Where the section sits once a write has found or placed it. Nothing else can move it while this setter holds the
  /// config.
  index: Option<usize>,
}

impl<'a> SectionSetter<'a> {
  pub fn new(ltx: &'a mut Ltx, section_name: String) -> SectionSetter<'a> {
    SectionSetter {
      ltx,
      section_name,
      index: None,
    }
  }

  /// Remove a field from this section, if the section and the field exist.
  pub fn delete(&'a mut self, key: &str) -> &'a mut SectionSetter<'a> {
    if let Some(index) = self.find() {
      self.ltx.sections[index].remove(key);
    }

    self
  }

  /// Get the entry in this section with `key`.
  pub fn get(&'a mut self, key: &str) -> Option<&'a str> {
    let index: usize = self.find()?;

    self.ltx.sections[index].get(key)
  }

  fn insert(&mut self, key: &str, value: &str) {
    let index: usize = self.place();
    let key: Arc<str> = index
      .checked_sub(1)
      .and_then(|previous| self.ltx.sections[previous].data.get_key_value(key))
      .map_or_else(|| Arc::from(key), |(held, _)| Arc::clone(held));

    self.ltx.sections[index].insert_shared(key, Arc::from(value));
  }

  /// Where the section is, if it is there yet.
  fn find(&mut self) -> Option<usize> {
    if self.index.is_none() {
      self.index = self.ltx.sections.get_index_of(self.section_name.as_str());
    }

    self.index
  }

  /// Where the section is, placed at the end of the config when it is not there yet.
  fn place(&mut self) -> usize {
    match self.find() {
      Some(index) => index,
      None => {
        let (index, _) = self
          .ltx
          .sections
          .insert_full(mem::take(&mut self.section_name), Section::default());

        self.index = Some(index);

        index
      }
    }
  }
}

impl LtxSectionWrite for SectionSetter<'_> {
  fn set<K, V>(&mut self, key: K, value: V) -> &mut Self
  where
    K: AsRef<str>,
    V: Display,
  {
    self.insert(key.as_ref(), SectionValueText::of(value).as_str());

    self
  }
}

#[cfg(test)]
mod test {
  use std::sync::Arc;

  use crate::binding::LtxSectionWrite;
  use crate::ltx::{Ltx, SectionSetter};

  #[test]
  fn writes_into_a_section_it_places_once_or_finds_where_it_stands() {
    let mut ltx: Ltx = Ltx::new();

    ltx.with_section("first").set("a", 1).set("b", 2);
    ltx.with_section("second").set("a", 3);

    let mut setter: SectionSetter<'_> = ltx.with_section("first");

    setter.set("a", 10).set("c", 30).delete("b");
    ltx.with_section("third").set_optional("a", None::<u32>);

    assert_eq!(ltx.sections().collect::<Vec<&str>>(), vec!["first", "second"]);
    assert_eq!(
      ltx.section("first").unwrap().iter().collect::<Vec<(&str, &str)>>(),
      vec![("a", "10"), ("c", "30")],
      "a replaced field to keep its place"
    );
  }

  #[test]
  fn reads_and_deletes_nothing_before_the_first_write_places_the_section() {
    let mut ltx: Ltx = Ltx::new();

    assert_eq!(ltx.with_section("absent").get("key"), None);

    ltx.with_section("absent").delete("key");

    assert!(
      !ltx.has_section("absent"),
      "neither a read nor a delete to create the section"
    );

    let mut setter: SectionSetter<'_> = ltx.with_section("absent");

    assert_eq!(setter.set("key", "value").get("key"), Some("value"));
  }

  #[test]
  fn shares_a_key_the_section_before_holds() {
    let mut ltx: Ltx = Ltx::new();

    ltx.with_section("0").set("position", "1,2,3").set("distance", 4);
    ltx.with_section("1").set("position", "5,6,7").set("other", 8);

    let keys = |name: &str| -> Vec<Arc<str>> {
      ltx.sections[name]
        .iter_shared()
        .map(|(key, _)| Arc::clone(key))
        .collect()
    };
    let (first, second): (Vec<Arc<str>>, Vec<Arc<str>>) = (keys("0"), keys("1"));

    assert!(Arc::ptr_eq(&first[0], &second[0]), "the repeated key to be stored once");
    assert_eq!(&*second[1], "other");
    assert_eq!(ltx.get_from("1", "position"), Some("5,6,7"));
  }
}
