use std::fmt::Write;

use specta::datatype::NamedDataType;

use crate::enumerations::enumeration_member::EnumerationMember;
use crate::enumerations::enumeration_subject::EnumerationSubject;
use crate::typescript::syntax::{render_docs, render_string};

/// One type's enum: what it stands for, and the members it declares in Rust declaration order.
pub(crate) struct Enumeration {
  pub subject: EnumerationSubject,
  pub members: Vec<EnumerationMember>,
}

impl Enumeration {
  /// The enum a type declares, which is its name under an `E` prefix.
  pub fn to_enum_name(type_name: &str) -> String {
    format!("E{type_name}")
  }

  /// Whether this enum may stand in for the type itself rather than only for one field of it.
  pub fn is_type_itself(&self) -> bool {
    self.subject == EnumerationSubject::Identities
  }

  /// This type as the frontend declares it: the enum, and the union it is declared with.
  ///
  /// An identities enum discards `union` because the union it wants is derived from the enum's own members rather
  /// than from Specta's list of literals; a discriminant enum keeps it, because the object shapes are the type.
  pub fn render(&self, named: &NamedDataType, union: &str) -> String {
    let name: String = Self::to_enum_name(&named.name);

    match &self.subject {
      EnumerationSubject::Identities => {
        let mut declaration: String = self.render_enum(&name, render_docs(&named.docs, named.deprecated.as_ref(), ""));

        let _ = writeln!(declaration);
        let _ = writeln!(
          declaration,
          "/** Every `{name}` as the spelling it crosses IPC as, for a value no member has narrowed. */"
        );
        let _ = writeln!(declaration, "export type {} = `${{{name}}}`;", named.name);

        declaration
      }
      // The union below carries the type's own documentation, so the enum says what it names instead of repeating it.
      EnumerationSubject::Discriminant { tag } => {
        let docs: String = format!(
          "/** Every `{tag}` the `{}` union is told apart by, so a switch or a comparison names one. */\n",
          named.name
        );

        format!("{}\n{union}", self.render_enum(&name, docs))
      }
    }
  }

  /// One `export enum` declaration under `docs`, so both subjects render their members the same way.
  fn render_enum(&self, name: &str, docs: String) -> String {
    let mut declaration: String = docs;

    let _ = writeln!(declaration, "export enum {name} {{");

    for member in &self.members {
      declaration.push_str(&render_docs(&member.docs, member.deprecated.as_ref(), "  "));

      let _ = writeln!(declaration, "  {} = {},", member.name, render_string(&member.value));
    }

    let _ = writeln!(declaration, "}}");

    declaration
  }
}
