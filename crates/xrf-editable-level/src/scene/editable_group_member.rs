/// One member of a group, `[object_N_ingroup_M]`, read as its own class's type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableGroupMember<T> {
  /// The holding group's `name`.
  pub group: String,
  pub object: T,
}
