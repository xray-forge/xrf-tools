use std::collections::BTreeMap;
use std::panic::Location;

use specta::Types;
use specta::datatype::NamedDataType;

use crate::enumerations::enumeration::Enumeration;
use crate::enumerations::enumeration_member::EnumerationMember;
use crate::enumerations::enumeration_subject::EnumerationSubject;

/// Every exported type that declares a TypeScript enum, keyed by the type's own name.
pub struct Enumerations {
  declared: BTreeMap<String, Enumeration>,
}

impl Enumerations {
  /// Resolves which collected types declare an enum, pairing the Rust variant names with the wire ones.
  ///
  /// Two views of one type are read together: `types` has been through the TypeScript format and carries the
  /// serialized spellings, while `collected` is the Rust source and carries the identifiers a member is named from.
  ///
  /// # Panics
  ///
  /// Panics when a mapped type has no Rust source, which means the two views can no longer be related at all.
  pub fn resolve(collected: &Types, types: &Types) -> Self {
    let sources: BTreeMap<(&str, Location<'static>), &NamedDataType> = collected
      .into_sorted_iter()
      .map(|named| ((named.module_path.as_ref(), named.location), named))
      .collect();
    let mut declared: BTreeMap<String, Enumeration> = BTreeMap::new();

    for named in types.into_sorted_iter() {
      let Some((subject, spellings)) = EnumerationSubject::of(named) else {
        continue;
      };

      // Located by declaration site rather than by name: a rename leaves the two views agreeing on nothing else.
      let source: &NamedDataType = sources
        .get(&(named.module_path.as_ref(), named.location))
        .unwrap_or_else(|| panic!("`{}` was mapped from no collected type", named.name));
      let members: Vec<EnumerationMember> = EnumerationMember::pair(named, source, spellings);

      EnumerationMember::assert_distinct(&named.name, &members);
      declared.insert(named.name.to_string(), Enumeration { subject, members });
    }

    Self { declared }
  }

  /// The enum `name` declares, or `None` for a type that declares none.
  pub fn enum_name_of(&self, name: &str) -> Option<String> {
    self
      .declared
      .contains_key(name)
      .then(|| Enumeration::to_enum_name(name))
  }

  /// Every type name whose enum may stand in for the type itself, mapped to it.
  ///
  /// A discriminant enum names one field of a union, not the union, so rewriting a parameter to it would ask a caller
  /// to pass `EArchiveSubject.WORLD` where a whole `ArchiveSubject` is wanted.
  ///
  /// What a command *parameter* is rewritten to, and nothing else. A parameter naming the union would accept a
  /// quoted literal the Rust side never declared, and there the enum costs nothing: the caller is writing the
  /// value, so it can name the member. A return is the opposite — a raw runtime string off the IPC channel,
  /// which an enum would assert about rather than check — so
  /// [`rewrite_parameter_type_references`](crate::references::rewrite_parameter_type_references)
  /// leaves returns spelled as the union.
  ///
  /// Struct fields in `types/` are deliberately **not** rewritten, which leaves 54 of them spelled as the union. That
  /// costs less than it looks: an enum member is assignable to the literal it equals, so
  /// `{ mode: EArchivePackMode.COMPRESS }` already compiles wherever `ArchivePackConfig` is built by hand, and only
  /// the *enforcement* is missing. Enforcing it is not closable here: a field has no direction of its own, only the
  /// struct does, and a struct can have both. `ArchivePackConfig` is returned by `default_pack_config` and taken by `pack` and `list_pack_volumes`
  /// — the frontend asks the backend for one, edits it in a form and sends it back — so whichever of the two
  /// spellings its `mode` were given would be wrong for the other half of that round trip. Closing it needs a
  /// per-direction type, which is a Rust-side split of a type the Rust side has one of, not a generator change.
  pub fn references(&self) -> BTreeMap<String, String> {
    self
      .declared
      .iter()
      .filter(|(_, enumeration)| enumeration.is_type_itself())
      .map(|(name, _)| (name.clone(), Enumeration::to_enum_name(name)))
      .collect()
  }

  /// One type as the frontend declares it: whatever Specta rendered, with the enum this type declares.
  ///
  /// The rendered union is taken rather than produced so that this one call answers for every type, enum or not, and
  /// a caller never has to know which shape it is holding.
  pub fn render(&self, named: &NamedDataType, union: &str) -> String {
    self
      .declared
      .get(named.name.as_ref())
      .map_or_else(|| union.to_owned(), |enumeration| enumeration.render(named, union))
  }
}
