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
  /// The static geometry, which the compiled level merged per sector, comes back as one object per sector, named after
  /// it and placed at the origin; sectors take a new sector's colour, and sectors and portals the editor's new names.
  SectorShells,
  /// Each static surface takes the game material of the collision faces under it, split where those differ, and a
  /// `shaders_xrlc.xr` compiler shader fitting what the level shows, as the sidecar's `geometry.surfaces` records.
  SurfaceDress,
  /// The collision faces no render triangle lies on come back as one object, `<level>_collision`, a mesh per sector,
  /// dressed as GSC dressed its collision-only props (`prop\prop_fake_kollision`, `def_shaders\def_trans_v`,
  /// `def_shaders\def_kolizion_vertex`).
  CollisionObject,
  /// `level.hom` comes back as `<level>_hom`, a surface of `materials\occ` as GSC's `labx8` keeps its occluders, and
  /// `level.som` as `<level>_som`, flagged a sound occluder, each polygon's game material the library's of the lowest
  /// number whose sound occlusion factor is the polygon's; both dressed with `prop\prop_fake_occ`.
  Occluders,
  /// Each ladder (`climable_object`) comes back as an object, `<level>_ladder_<N>`, of its box less the back face,
  /// standing at the middle of its lowest end and dressed as GSC's ladders (`prop\prop_fake_ladder`,
  /// `def_shaders\def_trans_v`, `def_shaders\def_kolizion_vertex`, `materials\fake_ladders` where the packet keeps no
  /// game material). Its packet is no spawn point, since the Build exports it again; the authored faces are lost.
  Ladders,
  /// Each breakable part (`meshes\brkbl#<N>.ogf`) comes back as a static object, `<level>_breakable_<N>`, where its
  /// `breakable_object` placed it, its fragments' bones dropped and each surface of its fragments' game material. Its
  /// packet is no spawn point, since the Build exports it again, cut into new random fragments.
  BreakableParts,
  /// Library objects found in the static geometry come back as scene objects referencing them at their best-fitting
  /// transform (one scale on every axis), their triangles taken out of the shells, each copied beside the scene's
  /// objects as the sidecar's `geometry.instances` records. Of two drawing the same geometry, the larger stands for
  /// both, then the one whose reference sorts first.
  LibraryInstances,
  /// Trees come back as multi-use objects, `<level>_tree_<N>`, one per set of subdivisions, dressed as their first
  /// placement lies on the collision form, with a scene object per placement. Impostor strips are cut from the level's
  /// atlas as `lod\lod_<model>` and its `_nm`, the model flagged `eoUsingLOD`; a model of no strip draws no impostor
  /// after a rebuild.
  TreeModels,
  /// Each detail model comes back as the library object drawing its geometry (the first by path), at the level's scale
  /// range and flags and a density factor of 1; the tool takes a density of 0.3, no base texture, colour indices or
  /// snap objects, and each slot's light is zeroed, as the Build zeroes it before lighting.
  DetailObjects,
  /// The AI map's nodes keep their cells, heights, planes and links, packed against a box of the map's floor and
  /// height; its generator settings but the patch size, and the editor's own settings, are a new tool's, since
  /// `level.ai` keeps none.
  AiMapSettings,
  /// The map's rectangle (`level.ltx`'s `bound_rect`) comes back as a level-bound shape, `level_bound_NNNN`, a box over
  /// it one metre tall around height 0; the file keeps only its x and z.
  MapBound,
  /// The `level.ltx` sections the Build does not write itself (a multiplayer map's anomaly sets) come back as the build
  /// options' text, which the Build writes back, without their blank and `;` lines.
  BuildOptions,
}
