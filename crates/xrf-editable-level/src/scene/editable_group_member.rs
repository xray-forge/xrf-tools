/// One member of a group, `[object_N_ingroup_M]` of the group tool's parts, read as its own class's type: the object and
/// the name of the group holding it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableGroupMember<T> {
  /// The holding group's own name, its `[object_N]`'s `name`.
  pub group: String,
  pub object: T,
}
