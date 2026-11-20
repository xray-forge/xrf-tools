use std::str::FromStr;

use xrf_enum_derive::{EnumDisplay, EnumFromStr, EnumVariants};

#[derive(Clone, Copy, Debug, PartialEq, EnumDisplay, EnumFromStr, EnumVariants)]
enum Language {
  #[display("eng")]
  English,
  #[display("ukr")]
  Ukrainian,
  Unknown,
}

fn main() {
  assert_eq!(Language::English.to_string(), "eng");
  assert_eq!(Language::Unknown.to_string(), "Unknown");

  assert_eq!(Language::from_str("ukr").ok(), Some(Language::Ukrainian));
  assert_eq!(Language::from_str("UKR").ok(), Some(Language::Ukrainian));
  assert_eq!(Language::from_str("unknown").ok(), Some(Language::Unknown));
  assert_eq!(
    Language::from_str("Ukrainian").map_err(|error| error.to_string()),
    Err(String::from("Parsing error: Invalid `Language` string representation: 'Ukrainian'"))
  );

  assert_eq!(
    Language::VARIANTS,
    &[Language::English, Language::Ukrainian, Language::Unknown]
  );

  for language in Language::VARIANTS {
    assert_eq!(Language::from_str(&language.to_string()).ok(), Some(*language));
  }
}
