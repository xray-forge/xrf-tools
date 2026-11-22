use std::borrow::Cow;

use specta::datatype::{DataType, Field, Fields, NamedDataType, Struct, Variant};

/// What a type's enum stands for, which is what decides where it is declared and what it may stand in for.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EnumerationSubject {
  /// The type is a closed set of identities. The enum *is* the type, and the union beside it derives from the enum.
  Identities,
  /// The type is a union of shapes told apart by one field. The enum names that field's values and nothing else, so
  /// it is declared before the union rather than in place of it.
  Discriminant { tag: String },
}

impl EnumerationSubject {
  /// What `named` declares an enum for, with the serialized spelling of each variant, or `None` for a type that
  /// declares none.
  ///
  /// The two shapes are mutually exclusive: a variant carrying data disqualifies a set of identities, and a variant
  /// carrying none cannot be internally tagged.
  pub(super) fn of(named: &NamedDataType) -> Option<(Self, Vec<&str>)> {
    if let Some(spellings) = Self::get_identity_spellings(named) {
      return Some((Self::Identities, spellings));
    }

    let (tag, spellings) = Self::get_tagged_spellings(named)?;

    Some((Self::Discriminant { tag: tag.to_owned() }, spellings))
  }

  /// The serialized spelling of every variant of `named`, when each is a bare identity and nothing else.
  ///
  /// A variant carrying data disqualifies its whole type, because the union Specta renders for it is a union of
  /// object shapes rather than of strings, and an enum of member names describes none of them. Such a type is offered
  /// to [`Self::get_tagged_spellings`] instead, which names its discriminants without claiming to name its values.
  fn get_identity_spellings(named: &NamedDataType) -> Option<Vec<&str>> {
    let Some(DataType::Enum(declared)) = named.ty.as_ref() else {
      return None;
    };
    let mut spellings: Vec<&str> = Vec::with_capacity(declared.variants.len());

    for (name, variant) in &declared.variants {
      if variant.skip {
        continue;
      }

      spellings.push(Self::get_identity_spelling(name, variant)?);
    }

    // An enum with no variant left renders as `never`, which no member list describes.
    (!spellings.is_empty()).then_some(spellings)
  }

  /// The tag field of `named` and the serialized spelling of every variant, when it is an internally tagged union.
  ///
  /// Read off the mapped shape rather than off Serde's container attributes, which `specta-serde` keeps to itself: an
  /// internally tagged variant is rendered as named fields whose tag field is a one-member literal carrying the
  /// variant's own spelling, or, for a newtype variant, as one field intersecting such named fields with its payload;
  /// nothing else renders that way. Externally tagged, adjacently tagged and untagged unions therefore fall out on
  /// their own, which matters because none of them has a discriminant to name.
  ///
  /// Every variant must agree on the tag field, since one enum is declared for the whole union.
  fn get_tagged_spellings(named: &NamedDataType) -> Option<(&str, Vec<&str>)> {
    let Some(DataType::Enum(declared)) = named.ty.as_ref() else {
      return None;
    };
    let mut spellings: Vec<&str> = Vec::with_capacity(declared.variants.len());
    let mut tag: Option<&str> = None;

    for (name, variant) in &declared.variants {
      if variant.skip {
        continue;
      }

      let (field, spelling) = Self::get_tag_fields(&variant.fields)
        .into_iter()
        .find_map(|fields| Self::get_tag_field(fields, name))?;

      if *tag.get_or_insert(field) != field {
        return None;
      }

      spellings.push(spelling);
    }

    tag.filter(|_| !spellings.is_empty()).map(|tag| (tag, spellings))
  }

  /// The named fields a variant's tag may stand among: its own, or those of each struct its one field intersects.
  fn get_tag_fields(fields: &Fields) -> Vec<&[(Cow<'static, str>, Field)]> {
    match fields {
      Fields::Named(named) => vec![named.fields.as_slice()],
      Fields::Unnamed(unnamed) => match unnamed.fields.as_slice() {
        [only] => match only.ty.as_ref() {
          Some(DataType::Intersection(parts)) => parts
            .iter()
            .filter_map(|part| match part {
              DataType::Struct(Struct {
                fields: Fields::Named(named),
                ..
              }) => Some(named.fields.as_slice()),
              _ => None,
            })
            .collect(),
          _ => Vec::new(),
        },
        _ => Vec::new(),
      },
      Fields::Unit => Vec::new(),
    }
  }

  /// The field of one variant carrying that variant's own spelling as a one-member literal, which is the tag.
  fn get_tag_field<'a>(fields: &'a [(Cow<'static, str>, Field)], variant: &str) -> Option<(&'a str, &'a str)> {
    fields
      .iter()
      .filter_map(|(field, declared)| {
        let DataType::Enum(literal) = declared.ty.as_ref()? else {
          return None;
        };

        match literal.variants.as_slice() {
          [(spelling, only)] if matches!(only.fields, Fields::Unit) => Some((field.as_ref(), spelling.as_ref())),
          _ => None,
        }
      })
      .find(|(_, spelling)| *spelling == variant)
  }

  /// The spelling one variant serializes as, or `None` where it carries data of its own.
  fn get_identity_spelling<'a>(name: &'a str, variant: &'a Variant) -> Option<&'a str> {
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
}

#[cfg(test)]
mod tests {
  use std::borrow::Cow;

  use serde::Serialize;
  use specta::datatype::{DataType, Enum, Field, NamedDataType, Variant};
  use specta::{Format, Types};
  use xrf_error::{XrfError, XrfResult};

  use super::EnumerationSubject;
  use crate::typescript::format::TypeScriptFormat;

  /// One enum datatype holding exactly these variants.
  fn enumeration(variants: Vec<(Cow<'static, str>, Variant)>) -> DataType {
    let mut declared: Enum = Enum::default();

    declared.variants = variants;

    DataType::Enum(declared)
  }

  /// The shape the TypeScript format maps a tag field to: a one-member literal carrying the variant's spelling.
  fn tag_literal(spelling: &'static str) -> DataType {
    enumeration(vec![(Cow::Borrowed(spelling), Variant::unit())])
  }

  /// A union as the mapped types hold one, built from `(spelling, tag field)` pairs.
  fn union(variants: Vec<(&'static str, Option<(&'static str, DataType)>)>) -> NamedDataType {
    let mut types: Types = Types::default();

    NamedDataType::new("Union", &mut types, |_, named| {
      named.ty = Some(enumeration(
        variants
          .into_iter()
          .map(|(spelling, tag)| {
            let variant: Variant = match tag {
              // A payload beside the tag, because an internally tagged variant that carries nothing else would be
              // a bare identity and belongs to the identities path instead.
              Some((field, literal)) => Variant::named()
                .field(field, Field::new(literal))
                .field("payload", Field::new(tag_literal("payload")))
                .build(),
              None => Variant::unit(),
            };

            (Cow::Borrowed(spelling), variant)
          })
          .collect(),
      ));
    })
  }

  #[test]
  fn an_internally_tagged_union_names_its_discriminants() {
    let named: NamedDataType = union(vec![
      ("thm", Some(("kind", tag_literal("thm")))),
      ("unsupported", Some(("kind", tag_literal("unsupported")))),
    ]);

    assert_eq!(
      EnumerationSubject::of(&named),
      Some((
        EnumerationSubject::Discriminant {
          tag: String::from("kind")
        },
        vec!["thm", "unsupported"]
      ))
    );
  }

  /// Bounds a newtype variant carries beside its tag.
  #[derive(Serialize, specta::Type)]
  struct Bounds {
    min: f32,
    max: f32,
  }

  /// A union of a bare, a newtype and a struct variant, internally tagged.
  #[derive(Serialize, specta::Type)]
  #[serde(tag = "kind", rename_all = "camelCase")]
  #[expect(dead_code, reason = "only its type is mapped")]
  enum Setting {
    Flag,
    Number(Bounds),
    Choice { values: Vec<String> },
  }

  #[test]
  fn an_internally_tagged_union_with_a_newtype_variant_names_its_discriminants() -> XrfResult {
    let mut collected: Types = Types::default();

    <Setting as specta::Type>::definition(&mut collected);

    let types: Types = TypeScriptFormat::default()
      .map_types(&collected)
      .map_err(|error| XrfError::new_serialization_error(error.to_string()))?
      .into_owned();
    let named: &NamedDataType = types
      .into_sorted_iter()
      .find(|named| named.name == "Setting")
      .expect("the union is mapped");

    assert_eq!(
      EnumerationSubject::of(named),
      Some((
        EnumerationSubject::Discriminant {
          tag: String::from("kind")
        },
        vec!["flag", "number", "choice"]
      ))
    );

    Ok(())
  }

  #[test]
  fn a_variant_whose_tag_does_not_carry_its_own_spelling_is_not_a_discriminant() {
    // An adjacently tagged union nests the payload under a second field, so the tag literal no longer sits beside
    // the variant's own name. Naming its arms as discriminants would describe a shape the frontend never sees.
    let named: NamedDataType = union(vec![("thm", Some(("kind", tag_literal("other"))))]);

    assert_eq!(EnumerationSubject::of(&named), None);
  }

  #[test]
  fn variants_disagreeing_about_the_tag_field_name_no_enum() {
    // One enum is declared for the whole union, so a union with two tags has no single discriminant to declare.
    let named: NamedDataType = union(vec![
      ("thm", Some(("kind", tag_literal("thm")))),
      ("unsupported", Some(("type", tag_literal("unsupported")))),
    ]);

    assert_eq!(EnumerationSubject::of(&named), None);
  }

  #[test]
  fn a_union_of_bare_identities_is_the_type_rather_than_a_discriminant() {
    let named: NamedDataType = union(vec![("thm", None), ("unsupported", None)]);

    assert_eq!(
      EnumerationSubject::of(&named),
      Some((EnumerationSubject::Identities, vec!["thm", "unsupported"]))
    );
  }
}
