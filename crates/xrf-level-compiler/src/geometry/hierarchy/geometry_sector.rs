/// One sector as the `level` file keeps it (`CSector::Save`): its root in the visual tree and the portals bounding it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeometrySector {
  /// Its root node, by index into [`crate::geometry::hierarchy::geometry_hierarchy::GeometryHierarchy::nodes`].
  pub root: u32,
  /// The portals with it on either side, by index into `build.prj`'s portals, ascending.
  pub portals: Vec<u16>,
}
