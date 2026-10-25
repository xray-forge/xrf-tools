/// Which count a descriptor belongs in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextureBumpVerdict {
  /// Nothing to bind, and nothing wrong with saying so.
  Undeclared,
  /// The engine binds a pair for this descriptor, each half either the declared file or a substitute.
  Bound {
    is_bump_missing: bool,
    is_companion_missing: bool,
  },
  /// The descriptor asks for a bump the engine never reads.
  InvalidDeclaration,
}
