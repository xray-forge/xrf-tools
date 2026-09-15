use std::borrow::Cow;
use std::collections::BTreeMap;

use specta::datatype::{DataType, Deprecated, NamedDataType, Variant};

/// One member of a generated TypeScript enum.
pub(crate) struct EnumerationMember {
  /// Screaming snake case of the Rust variant name, which is the name a frontend writes.
  pub name: String,
  /// The Rust variant this was derived from, retained so a collision can name what there is to rename.
  pub variant: String,
  /// Serialized spelling of the variant, which is the member's value and what actually crosses IPC.
  pub value: String,
  pub docs: Cow<'static, str>,
  pub deprecated: Option<Deprecated>,
}

impl EnumerationMember {
  /// The members of one enum, pairing each Rust variant with the spelling it serializes as.
  ///
  /// Paired by position because that is all the two views share: under `rename_all` the spelling appears nowhere in
  /// the Rust source, so a member's name and its value come from different places and only their order relates them.
  ///
  /// # Panics
  ///
  /// Panics when `source` is no longer an enum, or no longer declares as many variants as `named` serializes — either
  /// of which means the two views can no longer be paired at all.
  pub fn pair(named: &NamedDataType, source: &NamedDataType, spellings: Vec<&str>) -> Vec<Self> {
    let Some(DataType::Enum(rust)) = source.ty.as_ref() else {
      panic!(
        "`{}` renders as a list of identities but was collected as another shape",
        named.name
      )
    };
    let variants: Vec<&(Cow<'static, str>, Variant)> =
      rust.variants.iter().filter(|(_, variant)| !variant.skip).collect();

    assert_eq!(
      variants.len(),
      spellings.len(),
      "`{}` declares {} variants in Rust and serializes {}; they can no longer be paired by position",
      named.name,
      variants.len(),
      spellings.len()
    );

    variants
      .into_iter()
      .zip(spellings)
      .map(|((variant, declared), spelling)| Self {
        name: Self::to_member_name(variant),
        variant: variant.to_string(),
        value: spelling.to_string(),
        docs: declared.docs.clone(),
        deprecated: declared.deprecated.clone(),
      })
      .collect()
  }

  /// The TypeScript member name a Rust variant is written as.
  ///
  /// Derived from the Rust identifier rather than from the serialized spelling, because the spelling is a wire
  /// value and carries no word boundaries: upper-casing `AiCrow`'s spelling gives `AICROW` and `cform`'s gives
  /// `CFORM`. A boundary is either the end of a word, as in `PsStatic`, or the end of an initialism directly
  /// before one, as in `CForm`. Trailing underscores survive, so `S` and `S_` stay two members.
  pub fn to_member_name(variant: &str) -> String {
    let characters: Vec<char> = variant.chars().collect();
    let mut member: String = String::with_capacity(variant.len() + 4);

    for (index, character) in characters.iter().copied().enumerate() {
      let previous: Option<char> = index.checked_sub(1).map(|index| characters[index]);
      let next: Option<char> = characters.get(index + 1).copied();
      let starts_word: bool = character.is_ascii_uppercase()
        && previous.is_some_and(|previous| {
          previous.is_ascii_lowercase()
            || previous.is_ascii_digit()
            || (previous.is_ascii_uppercase() && next.is_some_and(|next| next.is_ascii_lowercase()))
        });

      if starts_word {
        member.push('_');
      }

      member.extend(character.to_uppercase());
    }

    member
  }

  /// Fails when two variants of one type fold to a single member name.
  ///
  /// Names the Rust identifiers rather than the wire spellings, because renaming is what resolves this and only the
  /// identifier can be renamed: under `rename_all` the two spellings appear nowhere in the Rust source, so a collision
  /// in `XrayExtension` would otherwise ask for `env_mod` and `envmod` to be renamed.
  pub fn assert_distinct(name: &str, members: &[Self]) {
    let mut claimed: BTreeMap<&str, &str> = BTreeMap::new();

    for member in members {
      if let Some(previous) = claimed.insert(&member.name, &member.variant) {
        panic!(
          "`{name}` variants `{previous}` and `{}` both name the member `{}`. Rename one of the Rust variants.",
          member.variant, member.name
        );
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::borrow::Cow;

  use super::EnumerationMember;

  /// A member as [`Enumerations::resolve`] builds one, for the guard that reads only the first three fields.
  fn member(variant: &str, value: &str) -> EnumerationMember {
    EnumerationMember {
      name: EnumerationMember::to_member_name(variant),
      variant: variant.to_string(),
      value: value.to_string(),
      docs: Cow::Borrowed(""),
      deprecated: None,
    }
  }

  #[test]
  fn a_word_starts_where_a_capital_follows_a_lower_case_letter() {
    assert_eq!(EnumerationMember::to_member_name("AiCrow"), "AI_CROW");
    assert_eq!(EnumerationMember::to_member_name("EnvMod"), "ENV_MOD");
    assert_eq!(EnumerationMember::to_member_name("PsStatic"), "PS_STATIC");
    assert_eq!(EnumerationMember::to_member_name("GeomX"), "GEOM_X");
  }

  #[test]
  fn a_word_starts_where_an_initialism_runs_into_one() {
    // The end of a run of capitals is a boundary only before a lower-case letter: `WSTM` is one word and `Gun` is
    // the next, so the break goes between them rather than before every capital.
    assert_eq!(EnumerationMember::to_member_name("WSTMGun"), "WSTM_GUN");
    assert_eq!(EnumerationMember::to_member_name("CForm"), "C_FORM");
  }

  #[test]
  fn a_word_starts_where_a_capital_follows_a_digit() {
    assert_eq!(EnumerationMember::to_member_name("GF1S"), "GF1_S");
    assert_eq!(EnumerationMember::to_member_name("SOG7B"), "SOG7_B");
    // Digits carry no boundary of their own, so a model number stays one word.
    assert_eq!(EnumerationMember::to_member_name("SM209"), "SM209");
    assert_eq!(EnumerationMember::to_member_name("Anm1"), "ANM1");
  }

  #[test]
  fn a_single_word_keeps_its_one_member() {
    assert_eq!(EnumerationMember::to_member_name("Thm"), "THM");
    assert_eq!(EnumerationMember::to_member_name("S"), "S");
  }

  #[test]
  fn a_trailing_underscore_survives_because_it_is_the_spelling() {
    assert_eq!(EnumerationMember::to_member_name("S_"), "S_");
    assert_eq!(EnumerationMember::to_member_name("Seq_"), "SEQ_");
  }

  #[test]
  fn a_member_is_derived_from_the_rust_identifier_and_not_from_the_wire_spelling() {
    assert_eq!(member("EnvMod", "env_mod").name, "ENV_MOD");
    assert_eq!(member("CForm", "cform").name, "C_FORM");
  }

  #[test]
  fn distinct_members_are_accepted() {
    EnumerationMember::assert_distinct("XrayExtension", &[member("EnvMod", "env_mod"), member("Ogf", "ogf")]);
  }

  #[test]
  #[should_panic(expected = "`XrayExtension` variants `EnvMod` and `Env_Mod` both name the member `ENV_MOD`.")]
  fn a_collision_names_the_rust_variants_rather_than_the_spellings_they_serialize_as() {
    EnumerationMember::assert_distinct(
      "XrayExtension",
      &[member("EnvMod", "env_mod"), member("Env_Mod", "envmod")],
    );
  }
}
