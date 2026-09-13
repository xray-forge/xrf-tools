//! Rendering a Rust enum whose variants carry no data as a TypeScript enum, beside the union it spells.
//!
//! A string union names no member, so every frontend use of one is a quoted literal that nothing checks
//! against the Rust source: a misspelling is a type error only if it is also not a member of the union, and a
//! renamed variant silently becomes a different valid string somewhere else. An enum gives each identity a
//! name the compiler resolves, which is what makes a rename fail at the call site rather than at runtime.
//!
//! Both are emitted, because they answer different questions. The enum is what a caller writes and what the
//! command signatures demand; the derived union is what a `Record<>` key, a fixture or a mock is written
//! against, where naming every member would be noise.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::panic::Location;

use specta::Types;
use specta::datatype::{DataType, Deprecated, Fields, NamedDataType, Variant};

/// One member of a generated TypeScript enum.
struct EnumerationMember {
  /// Screaming snake case of the Rust variant name, which is the name a frontend writes.
  name: String,
  /// Serialized spelling of the variant, which is the member's value and what actually crosses IPC.
  value: String,
  docs: Cow<'static, str>,
  deprecated: Option<Deprecated>,
}

/// Every exported type whose variants are all identities with no data, and the members each one declares.
pub(super) struct Enumerations {
  /// Exported type name to the members of the enum declared beside it, in Rust declaration order.
  members: BTreeMap<String, Vec<EnumerationMember>>,
}

impl Enumerations {
  /// Resolves which collected types become enums, pairing the Rust variant names with the wire ones.
  pub(super) fn resolve(collected: &Types, types: &Types) -> Self {
    let declared: BTreeMap<(&str, Location<'static>), &NamedDataType> = collected
      .into_sorted_iter()
      .map(|named| ((named.module_path.as_ref(), named.location), named))
      .collect();
    let mut members: BTreeMap<String, Vec<EnumerationMember>> = BTreeMap::new();

    for named in types.into_sorted_iter() {
      let Some(spellings) = unit_variant_spellings(named) else {
        continue;
      };

      let collected: &NamedDataType = declared
        .get(&(named.module_path.as_ref(), named.location))
        .unwrap_or_else(|| panic!("`{}` was mapped from no collected type", named.name));
      let Some(DataType::Enum(rust)) = collected.ty.as_ref() else {
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

      let declared_members: Vec<EnumerationMember> = variants
        .into_iter()
        .zip(spellings)
        .map(|((rust_name, variant), spelling)| EnumerationMember {
          name: to_member_name(rust_name),
          value: spelling.to_string(),
          docs: variant.docs.clone(),
          deprecated: variant.deprecated.clone(),
        })
        .collect();

      assert_members_are_distinct(&named.name, &declared_members);
      members.insert(named.name.to_string(), declared_members);
    }

    Self { members }
  }

  /// The enum `name` declares beside its union, or `None` for a type that stays a plain union.
  pub(super) fn enum_name_of(&self, name: &str) -> Option<String> {
    self.members.contains_key(name).then(|| to_enum_name(name))
  }

  /// Every type name that has an enum, mapped to it.
  ///
  /// This is what rewrites the IPC seam: a command signature naming the union would accept a quoted literal
  /// the Rust side never declared, which is the one place that has to stop compiling.
  pub(super) fn references(&self) -> BTreeMap<String, String> {
    self
      .members
      .keys()
      .map(|name| (name.clone(), to_enum_name(name)))
      .collect()
  }

  /// Declaration text for `named`, or `None` for a type Specta renders unchanged.
  pub(super) fn render(&self, named: &NamedDataType) -> Option<String> {
    let members: &Vec<EnumerationMember> = self.members.get(named.name.as_ref())?;
    let enum_name: String = to_enum_name(&named.name);
    let mut declaration: String = render_docs(&named.docs, named.deprecated.as_ref(), "");

    declaration.push_str(&format!("export enum {enum_name} {{\n"));

    for member in members {
      declaration.push_str(&render_docs(&member.docs, member.deprecated.as_ref(), "  "));
      declaration.push_str(&format!("  {} = {},\n", member.name, render_string(&member.value)));
    }

    declaration.push_str("}\n\n");
    declaration.push_str(&format!(
      "/** Every `{enum_name}` as the spelling it crosses IPC as, for a value no member has narrowed. */\n"
    ));
    declaration.push_str(&format!("export type {} = `${{{enum_name}}}`;\n", named.name));

    Some(declaration)
  }
}

/// The serialized spelling of every variant of `named`, when each is a bare identity and nothing else.
fn unit_variant_spellings(named: &NamedDataType) -> Option<Vec<&str>> {
  let Some(DataType::Enum(declared)) = named.ty.as_ref() else {
    return None;
  };
  let mut spellings: Vec<&str> = Vec::with_capacity(declared.variants.len());

  for (name, variant) in &declared.variants {
    if variant.skip {
      continue;
    }

    spellings.push(unit_variant_spelling(name, variant)?);
  }

  // An enum with no variant left renders as `never`, which no member list describes.
  (!spellings.is_empty()).then_some(spellings)
}

/// The spelling one variant serializes as, or `None` where it carries data of its own.
fn unit_variant_spelling<'a>(name: &'a str, variant: &'a Variant) -> Option<&'a str> {
  match &variant.fields {
    Fields::Unit => Some(name),
    Fields::Unnamed(fields) => match fields.fields.as_slice() {
      [only] => match only.ty.as_ref()? {
        DataType::Enum(literal) => match literal.variants.as_slice() {
          [(spelling, wrapped)] if matches!(wrapped.fields, Fields::Unit) => Some(spelling.as_ref()),
          _ => None,
        },
        _ => None,
      },
      _ => None,
    },
    Fields::Named(_) => None,
  }
}

/// The enum declared beside the union of one type, which is its name under an `E` prefix.
fn to_enum_name(name: &str) -> String {
  format!("E{name}")
}

/// The TypeScript member name a Rust variant is written as.
///
/// Derived from the Rust identifier rather than from the serialized spelling, because the spelling is a wire
/// value and carries no word boundaries: upper-casing `AiCrow`'s spelling gives `AICROW` and `cform`'s gives
/// `CFORM`. A boundary is either the end of a word, as in `PsStatic`, or the end of an initialism directly
/// before one, as in `CForm`. Trailing underscores survive, so `S` and `S_` stay two members.
fn to_member_name(variant: &str) -> String {
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
fn assert_members_are_distinct(name: &str, members: &[EnumerationMember]) {
  let mut claimed: BTreeMap<&str, &str> = BTreeMap::new();

  for member in members {
    if let Some(previous) = claimed.insert(&member.name, &member.value) {
      panic!(
        "`{name}` variants `{previous}` and `{}` both name the member `{}`. Rename one of the Rust variants.",
        member.value, member.name
      );
    }
  }
}

/// A JSDoc block in the shape Specta writes one, so a rendered enum reads like the rest of the file.
fn render_docs(docs: &str, deprecated: Option<&Deprecated>, indent: &str) -> String {
  if docs.is_empty() && deprecated.is_none() {
    return String::new();
  }

  if deprecated.is_none() {
    let mut lines = docs.lines();

    if let (Some(line), None) = (lines.next(), lines.next()) {
      return format!("{indent}/** {} */\n", escape_docs(line));
    }
  }

  let mut block: String = format!("{indent}/**\n");

  for line in docs.lines() {
    block.push_str(&format!("{indent} * {}\n", escape_docs(line)));
  }

  if let Some(deprecated) = deprecated {
    match deprecated
      .note
      .as_deref()
      .map(str::trim)
      .filter(|note| !note.is_empty())
    {
      Some(note) => block.push_str(&format!("{indent} * @deprecated {note}\n")),
      None => block.push_str(&format!("{indent} * @deprecated\n")),
    }
  }

  block.push_str(&format!("{indent} */\n"));
  block
}

/// Documentation text that cannot terminate the comment carrying it.
fn escape_docs(text: &str) -> Cow<'_, str> {
  match text.contains("*/") {
    true => Cow::Owned(text.replace("*/", "*\\/")),
    false => Cow::Borrowed(text),
  }
}

/// A wire spelling as a TypeScript string literal.
fn render_string(value: &str) -> String {
  format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
