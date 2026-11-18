use serde::{Deserialize, Serialize};

/// What a decompiled scene holds that its sources do not say, which the decompile chose; a part's inferences are
/// recorded only where it holds what they were chosen for.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelSceneInference {
  /// Objects are named as the editor names new ones, `<prefix>_NNNN`, the prefix their file's name or their tool's.
  ObjectNames,
  /// The level options are a new level's, built into and prefixed by the level's name, but for the light qualities,
  /// which the count of `build.lights`' generated sky and sun gives.
  LevelOptions,
  /// The level serves the game modes its respawn points answer to, or single player where it has none; made only where
  /// no `level.ltx` names them.
  GameTypes,
  /// The build parameters are the editor's defaults but for a weld distance of 0, for geometry already welded.
  BuildParameters,
  /// The camera stands at the origin, unturned.
  Camera,
  /// Sound sources take a new source's distances, which the editor reads again from each sound file.
  SoundSourceDistances,
  /// The wall mark tool takes a new tool's brush, and each decal the brush's size and turn.
  WallmarkBrush,
  /// Glows stand unturned with no fixed-size flag, which the compiled level does not keep.
  GlowTransformsAndFlags,
  /// Each light's brightness is the channel of its diffuse of the largest magnitude, sign kept, and its colour the rest;
  /// the compiled level keeps only their product.
  LightBrightness,
  /// A fuzzy light comes back as one light per point it was spread over.
  LightFuzzyPoints,
  /// A light only dynamic answers to `$static`; the compiled level keeps no control's name.
  LightControls,
  /// An entity's volume comes back as a shape attached to it at its position, unturned and unscaled, under a generated
  /// name; the spawn list keeps only the shapes.
  EntityShapes,
  /// The static geometry comes back as one object per compiled sector, named after it, its points in the level's space
  /// and placed at the origin; sectors take a new sector's colour, and sectors and portals are named as the editor
  /// names new ones. The compiled level merged the authored objects into each sector.
  SectorShells,
  /// Each static surface's game material is the collision face's its triangles lie on, a surface split where they lie
  /// on different ones or none; its compiler shader is one of `shaders_xrlc.xr` fitting what the level shows
  /// (collision, vertex lighting), chosen as the sidecar's `geometry.surfaces` records.
  SurfaceDress,
  /// The collision faces no render triangle lies on come back as one object, `<level>_collision`, a mesh per sector,
  /// dressed as GSC dressed its collision-only props (`prop\prop_fake_kollision`, `def_shaders\def_trans_v`,
  /// `def_shaders\def_kolizion_vertex`).
  CollisionObject,
  /// `level.hom` comes back as `<level>_hom`, a surface of `materials\occ` as GSC's `labx8` keeps its occluders, and
  /// `level.som` as `<level>_som`, flagged a sound occluder, each polygon's game material the library's of the lowest
  /// number whose sound occlusion factor is the polygon's; both dressed with `prop\prop_fake_occ`.
  Occluders,
  /// Each ladder the spawn list keeps (`climable_object`) comes back as an object, `<level>_ladder_<N>`, of its box
  /// less the back face (its front alone where the box is flat), standing at the middle of the box's lowest end and
  /// dressed as GSC dressed its ladders (`prop\prop_fake_ladder`, `def_shaders\def_trans_v`,
  /// `def_shaders\def_kolizion_vertex`), of `materials\fake_ladders` where the packet keeps no game material; its
  /// packet is no spawn point, since the Build exports it again. The authored faces are lost.
  Ladders,
  /// Each breakable part the Build wrote (`meshes\brkbl#<N>.ogf`) comes back as a static object,
  /// `<level>_breakable_<N>`, standing where its `breakable_object` placed it: its fragments' bones dropped, each surface
  /// of the game material of the fragments its faces follow, its compiler shader the editor's default. Its packet is no
  /// spawn point, since the Build exports it again, cut into new fragments at random.
  BreakableParts,
  /// Library objects found placed in the static geometry come back as scene objects placing them by reference, at the
  /// transform that best takes their rendered triangles onto the level's (one scale on every axis), their triangles
  /// taken out of the shells; each object is copied from its library with its thumbnail beside the scene's objects, as
  /// the sidecar's `geometry.instances` records. Where two library objects draw the same geometry, the larger,
  /// then the one whose reference sorts first, stands for both.
  LibraryInstances,
  /// Trees come back as multi-use objects, `<level>_tree_<N>`, one per set of subdivisions the compiled tree visuals
  /// draw, each in its own space and dressed as its first placement lies on the collision form, with a scene object per
  /// placement. A model's impostor strips are cut from the level's atlas where its first placement's `MT_LOD` names
  /// them, and written as the textures the Build loads (`lod\lod_<model>` and its `_nm`), the model flagged
  /// `eoUsingLOD`; a model of no strip draws no impostor far off after a rebuild.
  TreeModels,
  /// Each detail model comes back as the detail object of the object libraries drawing its geometry (the first by
  /// path where several do), at the level's scale range and flags and a density factor of 1; the tool takes a density
  /// of 0.3, no base texture, colour indices or snap objects, and the grid's box, and each slot's light is zeroed, as
  /// the Build zeroes it before the detail lighter lights it again.
  DetailObjects,
  /// The AI map's nodes keep their cells, heights, planes and links, packed against a box of the map's floor and
  /// height; its generator settings but the patch size, and the editor's own settings, are a new tool's, since
  /// `level.ai` keeps none.
  AiMapSettings,
  /// The map's rectangle (`level.ltx`'s `bound_rect`) comes back as a level-bound shape, `level_bound_NNNN`, a box over
  /// it one metre tall around height 0; the file keeps only its x and z.
  MapBound,
  /// The sections of `level.ltx` the Build does not write itself (a multiplayer map's anomaly sets) come back as the
  /// build options' text, which the Build writes into `level.ltx` again: a line each as the file held it, without its
  /// blank and `;` lines.
  BuildOptions,
}
