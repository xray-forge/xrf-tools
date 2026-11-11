# xrf-rawdata

Converts between the game's compiled assets and the SDK's rawdata, the tree the editors author in.

| Direction                            | Entry point                                  |
| ------------------------------------ | -------------------------------------------- |
| OGF → editable `.object`             | `ObjectDecompiler::decompile`                |
| `.dm` detail model → `.object`       | `ObjectDecompiler::decompile_detail`         |
| Level visuals → one static `.object` | `ObjectDecompiler::decompile_level_geometry` |
| Level files ↔ Level Editor parts     | `Level*Conversion::decompile` and `export`   |
| Level directory → Level Editor scene | `LevelSceneDecompiler::decompile`            |
| Level Editor scene → its game files  | `LevelSceneExporter::export`                 |
| Level Editor scene → `build.prj`     | `LevelBuildExporter::export`                 |
| `.object` → OGF                      | `ObjectCompiler::compile`                    |
| `.object` → `.dm`                    | `ObjectCompiler::compile_detail`             |
| Compare a compiled OGF with another  | `ObjectVerifier::verify`                     |

It sits beside the format crates because each conversion reads one format and writes another, so `xrf-ogf` and
`xrf-editable-object` stay independent of each other.

## Decompile an OGF

`ObjectDecompiler::decompile` turns an OGF into an `xrf-editable-object` `EditableObjectFile` the Actor Editor opens.

- Accepted: skeleton OGFs, and static ones (`MT_NORMAL`, `MT_PROGRESSIVE`, or an `MT_HIERRARHY` holding them).
- Refused: every other model type (trees, LODs, particles).

It undoes what the editor's `Export OGF` did:

**Surfaces and faces.**

- Every OGF child becomes a surface (texture and engine shader), taking only its full-detail sliding window.
- A face written twice on the same points with the same winding is dropped: the static export walks a two-sided
  surface's faces twice.
- A face written twice with the copy mirrored and its normals inverted becomes one face of a two-sided surface.

**Points and weights.**

- Corners split at seams and hard edges are welded back into points: same bits of position, same bones, weights equal
  within rounding. Two points the export's `VPack` moved onto one position stay two.
- Each vertex's links (padded to its child's layout) become the bones its point follows and their weights, normalised
  as the export left them, and from those one weight map per bone.

**Normals.**

- Per-face smoothing flags (`face_smoth_flags.h`) are recovered from where welded corners' normals agree, for the SDK.
- The corners' own normals go into `blender-xray`'s corner normals chunk, which the SDK does not read.

**Skeleton and motions.**

- Bones come from the bones and IK data chunks, with joint limits converted back from the physics engine's sense.
- The OGF's own motions become the object's: each bone keyed at every frame, rotations turned back into the angles the
  editor keys, plus how each motion plays and the bone parts it plays on.

A static OGF becomes a static object laid out as the SDK lays one out: object flags `0` (progressive when the OGF has
sliding windows), no bones, and one mesh whose UV maps are named `Texture`, with no weight map.

### Inferred values and provenance

What the OGF does not store is inferred and listed in `ObjectDecompile::inferred`:

- surface names, compiler shaders and game materials (the Actor Editor's `default`);
- the mesh name (`skin`, or `mesh` for a static object);
- a skeleton's bone lengths (the SDK's 0.5);
- smoothing and the two-sided flag;
- for an animated model, its motions' curves between frames, and for a motion holding one more key per bone than it
  declares (`blender-xray`'s, for an action starting between frames), its length from its keys.

`ObjectProvenance` is a sidecar written beside each decompiled file, so it is never mistaken for an authored one. It
records the source path and SHA-256, the tool, the time, every inference, and the tangent method the source was made
with (MikkTSpace when its description names Blender as the converter). `ObjectCompileOptions::following` compiles it
back with that method.

## Detail models

**Decompile.** `ObjectDecompiler::decompile_detail` turns a `.dm` (`xrf-level`'s `DetailModel`) into a static object
with one mesh and one surface: its engine shader and texture, and its triangles over positions and texture coordinates,
welded by position.

- A `.dm` stores no normals and no mirrored copy, so every edge comes back smooth, no surface is two-sided, and no corner
  normals are written.
- Its flags and scale range, which the Level Editor keeps on the `EDetail` entry rather than in the object, come back
  in `ObjectDecompile::detail` and the sidecar's `detail`.

**Compile.** `ObjectCompiler::compile_detail` writes a `.dm` as `EDetail::Update` and `Export` do (`compile/detail/`):

- one surface, from the faces of the first mesh in order;
- each corner takes the texture coordinate of its reference's first entry;
- a vertex within `EPS_L` of position and coordinate is reused, else appended;
- a face is dropped when two of its corners share a vertex, or it repeats a kept face in any order (mirrored copies
  included);
- flags and scale range come from `DetailParameters` (editor defaults `0` and `0.5..2`).

Verified: the SDK's 12 lightning sources compile byte for byte to vanilla's `.dm`, and each of vanilla's 12 lightning
`.dm` files decompiles and compiles back byte for byte.

## Decompile a level's visuals

`ObjectDecompiler::decompile_level_geometry` turns visuals of a compiled level into one static object, with the same
welding, duplicate dropping, two-sided folding and smoothing inference as an OGF.

- Each `LevelVisualGeometry` is one visual's texture and shader, its vertices from `level.geom`, and its full-detail
  indices, in the space they are stored in (a tree's own, until its placement is applied).
- Visuals sharing a texture and shader become one surface, and corners are welded across visuals, so faces that met in
  the source meet again.

`LevelVisualGeometry::read` resolves a visual: texture and shader from its own texture chunk or its shader table row,
its vertices, and its full-detail indices through `xrf-level`'s `read_full_detail_indices` (a static progressive
visual's own first window, or a progressive tree's from the `level.geom` table it names).

### Normal precision

A level stores each normal component in a byte, `(n + 1) * 127.5` truncated, so a two-sided face's mirror comes back
one step (1/127.5) off an exact negation. Level normals are therefore compared within one step; an OGF's float normals
within `1e-3`.

Over vanilla CoP's 22 levels at full detail: 11,692,886 mirrored corners are one step off and 36 exact. Corners sharing
a position agree exactly or differ by more than 20 steps, except 5,372 that are one step apart.

## A level's pass-through parts

The Level Editor's Build writes some level files straight from the scene's parts, with no compiler in between
(`BuilderGame.cpp`). Each such file decompiles back to the part it came from and exports again as the editor would.

- Each conversion takes the scene's typed part from `xrf-editable-level` and the compiled file from `xrf-level`.
- Every decompile takes the scene's `LevelDecompileContext`: the GUID its parts carry, and the names given so far.
- A compiled level keeps no object names. `LevelObjectNames` names a scene's objects `<prefix>_0000` upwards, as the
  editor numbers new objects; `get_next_after_file` takes the prefix from the file an object plays or shows.

| Conversion                        | Files                                                         | Round trip                                                     |
| --------------------------------- | ------------------------------------------------------------- | -------------------------------------------------------------- |
| `LevelParticleConversion`         | `level.ps_static` ↔ `ps.part`                                 | All 18 of vanilla CoP's files, byte for byte                   |
| `LevelSoundSourceConversion`      | `level.snd_static` ↔ `sound_src.part`                         | 91 files (1,303 sounds) from memory; all but one through text  |
| `LevelSoundEnvironmentConversion` | `level.snd_env` ↔ `sound_env.part`                            | 15 of 20 byte for byte; 5 within 6.1e-5                        |
| `LevelFogVolumeConversion`        | `level.fog_vol` ↔ `fog_volume.part`                           | All 174 files through text                                     |
| `LevelWallmarkConversion`         | `level.wallmarks` ↔ `wallmark.part`                           | All 74 files through the binary part; `labx8`'s part exact     |
| `LevelWayConversion`              | `level.game` paths ↔ `way.part`                               | 202 files from memory; all but two through text                |
| `LevelAiMapConversion`            | `level.ai` ↔ `ai_map.part`, `build.aimap`                     | `labx8`'s 6,781 nodes at GSC's part's cells, every link alike  |
| `LevelDetailConversion`           | `level.details` ↔ `detail_object.part`, `build.details`       | `mp_darkvalley` built back: objects and slots as shipped       |
| `LevelSpawnPointConversion`       | `level.game` respawn points, `level.env_mod` ↔ `spawn.part`   | 200 of 202 levels through text                                 |
| `LevelEntityConversion`           | `level.spawn`, `level_rs.spawn` ↔ `spawn.part`                | Every packet read comes back; floats off the `%f` grid rounded |
| `LevelGlowConversion`             | `level`'s glows → `glow.part`                                 | 4,563 glows over 202 levels; `labx8`'s 29 exactly GSC's        |
| `LevelLightConversion`            | `build.lights`, `level`'s lights → `light.part`               | Built back exactly over all 186 levels with `build.lights`     |
| `LevelGeometryConversion`         | `level`, `level.geom` → objects, `sector.part`, `portal.part` | Every static triangle of vanilla CoP's 22 levels accounted for |

### Particles

- `decompile` gives each placement an object named after its effect, at the saved position, angles and scale that
  compose into its matrix (`EditableObjectTransform::from_matrix_as_saved`).
- `export` writes what `EParticlesObject::ExportGame` does, and no file for a part without particles, as the build
  writes none.
- Exception: a particle pitched straight up or down. There the editor's `getHPB` takes no bank, so the saved angles do
  not recompose exactly and the matrix comes back a few units in the last place off.

### Static sounds

- `decompile` gives each sound an object named after its file, looped, with the distances a new sound source takes (the
  editor reads the file's own on load, and the export drops them).
- Each play window is stored in seconds whose `%f` text exports to the same milliseconds. `u32(seconds * 1000)`
  truncates, so 511 ms saves as `0.511001`.
- `export` writes what `ESoundSource::ExportGame` does, and no file for a part without sounds.
- The one file that does not round-trip through text is SGM 2.2's `l04u_labx18`: its positions are off the `%f` grid, as
  a build from an unsaved editor session would be, and the text rounds them within a unit of the sixth decimal.

### Sound environments

The CoP SDK writes no `level.snd_env` (its `MakeEnvGeometry` is commented out, as in IX-Ray's editor), so `export`
ports that code:

- environments in the order volumes first name them, inner first;
- each exported volume's 8 box corners, merged within `EPS` as `CDB::Collector::VPack` merges them, and 12 faces;
- bounds from every volume's `GetBox`, including volumes skipped for equal or empty environments.

`decompile` refuses a mesh not laid out that way. Each box becomes a volume named `sound_env_NNNN`: the editor's unit
box placed by a saved transform whose corners give the same bits (`EditableObjectTransform::find_saved`), or else at
the origin with its own saved box matrix.

Of the 20 shipped files, 15 (GSC's `l12u_sarcofag` among them) round-trip byte for byte. Five of GAMMA's hold boxes no
saved text can produce; they come back within 6.1e-5 with the same environments and faces.

### Fog volumes

`export` writes what `BuildGame` does for every level:

- version 3;
- per group, in ascending number with the ungrouped last: the emitter's profile line and transform, then its occlusion
  volumes' transforms;
- a group that is not one emitter plus occlusions is refused, as the build fails on it;
- occlusions keep the part's order, which the editor's `std::sort` by type leaves unspecified.

`decompile` numbers each body's group from 1: its emitter and one occlusion volume per obstacle, named
`fog_volume_NNNN`, each at the saved transform that composes into its matrix.

All 174 files across the game trees round-trip through the part's text: 172 with no body, plus CS's
`agroprom_underground` (4 bodies, 144 obstacles) and SGM 2.2's copy.

### Wall marks

- `decompile` gives the wall mark tool a new tool's brush and a slot per stored slot; each decal keeps the file's sphere
  and takes the box `AddWallmark` gives its fan, no selection (as every decal GSC saved) and the brush's size.
- `export` writes what `ESceneWallmarkTool::Export` does from the tool as the editor loaded it: fans past 512 vertices
  dropped, then the slots that leaves empty.
- `labx8`'s own `wallmark.part` rewrites and exports byte for byte, and its 31 decals match the decompile in flags,
  boxes, spheres and fans; only their recorded size differs.

### Patrol paths

- `decompile` gives each path a way under its name, its points unselected and each link on the point it leaves, in the
  file's order; a type chunk, which none ships, becomes the way's type.
- `export` writes what `CWayObject::ExportGame` does: points, then links point by point, and no type chunk.
- `level.game`'s respawn points come from the spawn tool; CoC's `jupiter` and SGM 2.2's `l04u_labx18` hold positions off
  the `%f` grid, which the text rounds.

### Respawn points and environment modifiers

- `decompile` gives each respawn point and each environment modifier a spawn point at the file's position (and turn),
  named after its kind; a colour comes back as the packed word whose channels `u32_3f` gives the same float bits from.
- `export_respawn_points` and `export_environment_modifiers` write what `CSpawnPoint::ExportGame` does, the latter
  under `SPAWNPOINT_VERSION` for every level.
- 4,951 respawn points and 25 modifiers over every tree; CS's `mp_training_camp` holds positions off the `%f` grid, and
  SGM 2.2's `l04u_labx18` an older `env_mod` that keeps no flags.

### Entity spawn data

`LevelSpawnDataFields` is an entity's `[object_N_spawndata]` as the editor saves it: `write_section` numbers the
`ChunkField`s a spawn packet records from `000001` and prints each as `CInifile` prints its kind (`%d` integers, a
`u32` past `i32::MAX` negative, `%f` floats, `x, y, z` vectors, quoted strings), refusing a string holding `"`, which
`SIniFileStream` writes unescaped. As a `ChunkFieldSource` it reads them back as the packet asks; `assert_read` refuses
a field past the packet's last.

### Entities

`LevelEntityConversion` converts the server entities between `spawn.part` and the spawn lists the build writes:
`level.spawn`, and `level_rs.spawn` for entities marked to respawn in multiplayer (`fl` 1).

- `export` writes what `CSpawnPoint::SSpawnData::ExportGame` does. It reads each entity's spawn data as its section's
  class, then takes the name, position and angles from the spawn point. A class with a volume (restrictors, zones,
  smart terrains, level changers, climbable objects, smart covers) takes its shapes from the attached shape with that
  shape's scale applied, and is refused without one, as the build refuses it.
- `decompile` gives each packet a spawn point at its position and angles under its name, its spawn packet as spawn
  data, and a class's volume as a shape attached at the entity, unturned. Packet names are reserved first, so no
  generated name takes one.
- A packet `xrf-spawn` cannot read is left out and returned with its reason; the scene's sidecar lists it. These are
  multiplayer sections the section table does not map, Clear Sky and Shadow of Chernobyl packets, and SGM 2.2's own
  classes.

Over every game tree's spawn lists, each packet read comes back byte for byte, or with only floats off the `%f` grid
rounded, as the editor's own save rounds them (mostly climbable and breakable shapes). GSC's `labx8` scene exports its
318 entities byte for byte against the shipped `level.spawn`.

Breakable and climbable objects are built by the editor from scene geometry, so where the geometry is decompiled they
come back as objects instead and leave `spawn.part` (`LevelEntitySources::folded`; see the geometry below).

#### From a root spawn

A game starts from its root spawn (`all.spawn`), not from `level.spawn`, and modders often edit the root spawn after it
was built. Given a root spawn's objects standing on the level (`LevelEntitySources::root`, which
`LevelDecompileOptions::root_spawn` reads), the decompile takes them instead:

- Each takes the place of its `level.spawn` packet of the same name, in `level.spawn`'s order. Objects only the root
  spawn holds follow in its order.
- The fields xrAI fills in when it builds the root spawn (`AlifeBuiltFields`: the spawn id, game and level vertices,
  distance, a level changer's destination, a zone's artefact spawn offset) come from that `level.spawn` packet, since
  xrAI overwrites whatever the editor wrote there. An object `level.spawn` lacks takes a fresh entity's.
- Graph points come from `level.spawn`: a root spawn keeps them as graph vertices.
- A `level.spawn` packet the root spawn lacks is left out.

The sidecar's `rootSpawn` records the file and its hash, and each disagreement: an object only one of the two holds, or
whose packet differs. It also keeps each update packet that differs from a fresh entity's (uninitialised physics states,
edited conditions) as hex, since the scene keeps spawn packets alone.

Over vanilla's `all.spawn`, all 6,464 objects agree with their `level.spawn` packets. Every other root spawn differs
from its own spawn lists where it was edited; each difference is a recorded disagreement.

### Glows

Glows are not passed through: the build hands each to the level compiler as a `b_glow` (`SceneBuilder::BuildGlow`),
which keeps its position, radius and the shader table row of its shader and texture in the compiled `level`
(`fsL_GLOWS`). `decompile` gives back a glow per record, unturned, named after its texture, with no fixed-size flag,
which the compiled level drops. Their export goes into `build.prj` (`LevelBuildExporter`).

- GSC's `labx8` holds its 29 glows inside groups: each matches a decompiled glow at distance 0 with the same radius,
  shader and texture.
- 198 of the 202 levels round-trip through the part's text; four (CS's `mp_autostation`, `mp_bath`, `mp_garbage` and
  SGM 2.2's `l04u_labx18`) hold positions off the `%f` grid, which the text rounds within 7e-7.

### Lights

Lights are not passed through either. The build hands each to the level compiler as a static light, a dynamic one or
both. The compiler writes the static ones into `build.lights` by their control's layer (chunk 0 for `$static`, 1 for
`$hemi` beside the sky it generates), each diffuse divided by its energy and its range times 1.1, and the dynamic ones
into `level`'s `fsL_LIGHT_DYNAMIC` as given.

`decompile` gives back:

- a light per compiled point light of an authored layer, under that layer's control;
- one light affecting both static and dynamic lighting where a dynamic light stands at the same position with the same
  diffuse;
- a light under `$static` for each remaining dynamic point light.

What the compiled files keep none of is inferred and recorded:

- brightness is the diffuse channel of the largest magnitude, sign kept so a darkening light stays one, and the colour
  the rest;
- a fuzzy light comes back as one light per point;
- the range is the float whose 1.1 times gives the compiled bits.

GSC's `labx8` lights, built as the editor and compiler build them, give its compiled ones exactly, and every level's
decompiled lights build back to its own exactly.

The sun and sky the build generates fold back into the scene's settings:

- `get_hemi_quality` reads the level options' hemisphere quality from chunk 1's generated sky (1, 26, 91 or 196 lights
  for 0 to 3);
- `get_sun_quality` reads the sun quality from chunk 2's samples (1, 9, 16 or 49);
- the light tool's `sun_shadow_dir` is the dynamic sun's heading and pitch, its heading within one turn up from 0.

`labx8` gives back GSC's own hemisphere quality 3, sun quality 0 and sun direction text. CS-era suns of other sample
counts keep a new level's quality, and a direction may come back one unit off in its sixth decimal.

### A whole scene

`LevelSceneDecompiler::decompile` turns a level's directory into a scene (`LevelSceneDecompile`) in steps over one
draft, each a `LevelSceneStep` declaring the files it reads (`level/scene/steps/`):

| Step                 | Files                                                                                      | Gives                                            |
| -------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------ |
| `LevelGeometryStep`  | `level`, `level.geom`, `level.cform`, `level.hom`, `level.som`, `level.wallmarks`, `level.spawn`, `meshes/brkbl#*.ogf` | Objects, scene objects, sectors, portals (below) |
| `LevelBundleStep`    | `level`, `build.lights`                                                                    | Glows, lights, the light qualities               |
| `LevelCompanionStep` | `level.ps_static`, `level.snd_static`, `level.snd_env`, `level.fog_vol`, `level.wallmarks` | One part each                                    |
| `LevelGameStep`      | `level.game`, `level.env_mod`, `level.spawn`, `level_rs.spawn`                             | Ways, spawn points, the game modes               |

The decompile reads every step's files once (`LevelSceneSources`, which also reads the bundle once for the two steps
that need it), runs the steps in that order, so names come out the same each time, and finishes the draft:

- a part per tool, sorted by name, each recording the files it came from;
- a main file of a new level named after `LevelDecompileOptions::name`, serving its respawn points' game modes (single
  player where it has none), the editor's build parameters with a weld distance of 0, the camera at the origin, and
  recording the files its options came from;
- a scene GUID from the SHA-256 over each source's name, length and bytes in name order, then a root spawn's hash where
  the entities came from one, so the same level always gives the same scene;
- the objects the scene places (`LevelSceneObjectFile`), each with its own `ObjectProvenance` naming the compiled file
  it came from (`LevelDecompiledObject::source`).

A new source is a file in a step's `FILES`, or a `LevelSceneListing` in its `LISTINGS` for files known by what their
names hold (the breakable parts), read in its `decompile`; a new kind of part is a step of its own. A step that needs a
file another step decompiles reads it without taking it (`LevelSceneSources::peek_read`).

`LevelGeometryConversion::decompile` gives a level's geometry back from `LevelGeometrySources` (the bundle, its
visuals, `level.geom`, and where held the indexed collision form, `level.hom`, `level.som`, the wall marks, the spawn
list and the breakable parts): every object below, each placed by a scene object, with the sectors and portals. The static geometry comes back as one object
per compiled sector, `levels\<name>\<name>_sector_<N>`: the static visuals the sector's root reaches, trees and their
impostors left out, through `ObjectDecompiler::decompile_level_geometry`. Each object keeps the level's coordinates and its scene object
stands at the origin, since a pivot inside a sector-sized object would round the low bits off the points nearer the
origin and open seams between sectors. A sector per compiled sector names its object, and a portal per compiled portal
joins two of them; the Build numbers sectors in the order the scene lists them, so the compiled indices come back. A
sector drawing no static geometry would leave no object to name, and the editor drops a sector of no item, renumbering
the rest: such a level keeps its shells but no sector or portal, each empty sector refused. A sector drawing trees
alone is kept, naming their placements.

Trees come back as the multi-use objects the Build placed them from (`LevelTreeConversion`, `BuildMUObject`). xrLC
draws each placement as one tree visual per subdivision of its model, in the model's own space with the placement's
matrix, under one `MT_LOD` where the model has an impostor. So a placement is the visuals one `MT_LOD` parents, or that
share a sector and matrix where none does, and a model is the set of subdivisions a placement draws; no subdivision
belongs to two models in vanilla's `mp_darkvalley`, `mp_firestation` or `zaton`. Each model is written once as
`levels\<name>\<name>_tree_<N>`, in its own space and flagged `eoMultipleUsage`, dressed from where its first placement
lies on the collision form; each placement is a scene object of its matrix that its sector names. `mp_darkvalley`'s 162
tree visuals come back as 19 models and 73 placements.

A model's impostor comes back where its first placement's `MT_LOD` names it in the level's atlas,
`level_lods.dds` and `level_lods_nm.dds` (`LevelGeometrySources::impostors`, a `LevelTreeImpostorAtlas`). The eight
facets' coordinates must fit one place of a whole 512 by 64 strip, turned or not; the strip and its normals are cut back
out as the textures the Build loads, `lod\lod_<model>` and its `_nm` under `$game_textures$`
(`LevelGeometryDecompile::textures`, written by `LevelSceneDecompile::write_to_path` under the textures root), and the
model is flagged `eoUsingLOD`. The Build stands the quads in the union of the model's meshes' boxes, which the editor
reads from the file as saved; the source object's box held geometry the level does not draw, such as a collision trunk
under the ground, so the model's stored box is widened to the one the facets were stood in, read back through the
placement. An impostor that does not cut is refused and its model comes back without one. All 19 of `mp_darkvalley`'s
cut, and rebuilt they stand within 0.05 mm of the shipped quads, sampling the same pixels but for DXT noise.

With a `LevelSurfaceInference` (decision 14, `LevelSurfaces::Infer`), each visual is split by the game material its
triangles lie on before the decompile, so each surface carries one `SurfaceDress` (compiler shader and game material,
the editor's `default` otherwise):

- the game material is the collision face's on the triangle's very points, else that of the one its normal through its
  centre crosses nearest it within 5 cm (xrLC splits render faces it keeps whole in `level.cform`; the nearest, not the
  first a ray from above meets, keeps a floor's material under a plate lying on it), else none; `level.cform` stores
  the material's `gamemtl.xr` ID, as the editor wrote it into `build.prj`;
- the compiler shader is one of `shaders_xrlc.xr` that renders and fits the evidence, collision and vertex lighting
  (vertices without lightmap coordinates), ranked by how often an object library pairs it with the surface's engine
  shader (`LevelShaderPairings`), then the one GSC's objects use most for that evidence, then library order;
- `LevelSurfaceCensus` counts exact, traced and uncollided triangles and lists every choice with its reason.

Where the level holds `level.cform`, `LevelCollisionIndex` holds those lookups and marks which collision faces a static
render triangle, or only a tree's, lies on. The faces nothing covers come back as `levels\<name>\<name>_collision`, a
mesh per sector they lie in (`collision_<N>`, which that sector names), dressed as GSC dressed its collision-only props
(`LevelGeometryConversion::COLLISION_TEXTURE` and the two shaders beside it). `ObjectDecompile::merge_meshes` joins the
sectors' decompiles into the one object. A face no render triangle's centre lands on is still covered where it lies
under one of its sector's render triangles that lies on collision of its game material (`cover_under_static`): xrLC
kept it whole where it split the render faces over it. A face of another material there is collision of its own, such
as a collision-only prop laid on a floor, and stays. Over vanilla, 288,343 of 10,830,486 collision faces are
collision-only and 2,646,858 lie under trees alone.

The Level Editor builds a picking model of every mesh it loads and refuses one of fewer than 4 points and 2 faces
(`CDB::MODEL::build`), so `ObjectDecompiler::decompile_level_geometry` gives a mesh of one face back halved through its
longest edge, the same area in two faces, and records `ObjectInference::HalvedFace`; the census counts it as the one
face it was. A sector's lone collision-only face is the usual case (`labx8`'s sector 13).

The Build adds every wall mark to the static faces too (`ESceneWallmarkTool::ExportStatic`, compiler shader
`def_shaders\def_vertex_ghost_no_shadow`) beside writing `level.wallmarks`, so the compiled geometry holds each decal
twice. With the wall marks, `LevelDecalIndex` drops a static triangle of a slot's texture whose centre lies within 1 cm
of one of that slot's decal triangles: `wallmark.part` holds it, and a rebuild would otherwise draw it twice. Over
vanilla every one of the 11,768 static triangles of a slot's texture lies so (6,601 exactly), and the census counts them
as `decal_triangles`.

xrLC leaves needles along T-junctions in its own output: triangles a millimetre or two across whose two shorter edges add
up to the longest within a few single-precision steps. The Build and xrLC measure a face by Heron's formula in single
precision, which rounds such a face's area to zero, and xrLC stops on one (`Face::Verify`), so the decompile leaves
them out, static and collision-only alike, and the census counts them as `needle_triangles` (54 on `labx8`).

`LevelOccluderConversion` gives `level.hom` back as `<name>_hom`, an ordinary object whose surfaces have the
`materials\occ` game material, as GSC's `labx8` keeps its occluders, and `level.som` as `<name>_som`, flagged
`eoSoundOccluder`, a surface per game material whose sound occlusion factor is a polygon's (the lowest number where
several share it). A two-sided polygon goes in with its mirror, so the decompile folds the two into one two-sided face.
Exported again as the Build exports them, every vanilla `level.hom` polygon comes back, and every `level.som` polygon
of CS, CoC, Anomaly and GAMMA.

The Build leaves faces of a climbable or breakable game material out of the geometry and exports each connected part as
an entity, so with the spawn list (`LevelGeometrySources::spawn`) the decompile folds them back into objects and names
their packets in `LevelGeometryDecompile::folded_entities`, which the entities then leave out:

- `LevelLadderConversion` turns a `climable_object` into `<name>_ladder_<N>`: its box less the face behind its thinnest
  axis (the front alone where the box is flat), standing at the middle of the box's lowest end, dressed as GSC dressed
  its ladders, of the packet's game material or `materials\fake_ladders` where the packet keeps none.
- `LevelBreakableConversion` turns a `breakable_object` into `<name>_breakable_<N>` from the part its visual names,
  `meshes/brkbl#<n>.ogf` (`LevelGeometrySources::breakable_parts`), through `ObjectDecompiler::decompile_breakable`: the
  random fragments' bones dropped and their points welded, each surface of its fragments' game material. A breakable
  whose part is missing stays an entity, counted.

Both stand where their packets placed them, `Affine3d::from_xyz` of the angles. Over vanilla, all 380 ladders and 173
breakables fold back.

Over vanilla CoP's 22 levels, 7,189,838 static triangles lie exactly on a collision face, 2,036,780 are traced to one
and 601,675 lie on none. Where the SDK's objects dress the same texture and engine shader, the compiler shader is one
GSC used for 95.2% of faces (96.0% with the SDK's objects as the library) and the game material for 86.8% (the
collision object counted).
`LevelGeometryCensus` counts the triangles read, the faces written, the two-sided folds, the repeats dropped, and the
tree triangles, models and placements. Over every tree's 202 levels, every static triangle is a face, a fold or a repeat, and every
written point is a position the level's vertices hold, bit for bit.

`write_to_path` writes the sidecar, `<name>.level.xrf.json` (`LevelSceneProvenance`), then each object under the
objects root after its own sidecar, then the textures the objects need (the impostor strips) under the textures root,
then the scene, so a failed write never leaves a decompile that looks authored.
The sidecar holds the front-end the options name, each source's SHA-256, each written file and its sources, and what
the decompile chose rather than read (`LevelSceneInference`), recorded only for a part that holds what it was chosen
for: a level of no light records no light inference. The object and scene sidecars share one convention, the
`.xrf.json` suffix beside their file and sources named by SHA-256.

`LevelSceneExporter::export` goes back: the files the Build writes from a scene's parts (`BuildGame`,
`BuildWallmarks`), by name. `level.game`, `level.env_mod` and `level.fog_vol` are written for every scene, empty where
it has no such tool; the rest only where their tool exports something. All 22 vanilla CoP levels decompile and export
every such file byte for byte. Across the other trees the differences are the ones each conversion above names, plus
nine Anomaly and GAMMA underground levels that ship no `level.env_mod` or `level.fog_vol`, which CoP's Build always
writes. `write_to_directory` writes them, each staged,
saying of each whether it is new, the same or changed. Since the editor's Build moves every file it may write aside
first, it refuses a directory holding one the scene does not build rather than delete it. Whether existing files may be
replaced is the front-end's to ask (`--force`, a confirmation), as `LevelSceneDecompile::get_existing_paths` lets it ask
before a decompile and `get_existing_object_paths` after it. Vanilla CoP's `labx8`, `zaton` and `mp_atp`,
and CS's `agroprom_underground`, decompile, read back to the same bytes, and export every file byte for byte; so does
GSC's own `labx8` scene.

Every export reads a tool's objects with the groups' members of its class first (`EditableGroupTool`), as the editor
loads the group tool before the others and keeps a member's transform in the level. GSC's `labx8` holds 99 scene
objects, 229 lights, 43 spawn points and 29 glows in groups; with them its scene exports 361 of the shipped
`level.spawn`'s 379 packets byte for byte and in its order, the other 18 being the climbable objects the Build cuts out
of the geometry, which no export here makes yet.

`LevelBuildExporter::export` writes the `build.prj` the Build hands xrLC (`CompileStatic`, `SaveBuild`), reading each
placed object from an objects root and its game materials from `gamemtl.xr`: the sky and the sun generated from the
level options (the sun's samples each taken from the one before, as the shipped editor's loop reads the variable it
declares), then glows, static objects (`BuildMesh`: world space, occluder, breakable and climbable surfaces skipped,
faces of no area dropped, two-sided backs added), the scene's lights under their controls, portals and wall marks,
each material once per texture, shaders and sector. A wall mark's triangle of no area is left out too, where the
editor passes it on: xrLC refuses it by the same measure and stops on seven, as vanilla `jupiter`'s wall marks hold. A portal of more than six corners is simplified as `CPortal`
simplifies one (`LevelBuildPortal`: the plane fitted, the convex hull, the smallest edges collapsed), in double
precision rather than bit for bit. Beside it go `level.hom` and `level.som` (`BuildHOMModel`: an `eoHOM` object's faces
and any `materials\occ` surface's; `BuildSOMModel`: an `eoSoundOccluder` object's, with its game materials' factors)
and `level.ltx` (`LevelMapConversion::export`); `LevelBuildExport::into_files` gives them by name. The ladders it cuts
from the geometry (`LevelBuildClimbables`, `ExportClimableObjects`: climbable faces in parts sharing a point, each a
`climable_object` at its box) are `LevelBuildExport::entities`, which `LevelSceneExporter::export` writes first in
`level.spawn`; over GSC's `labx8` they are SDK 0.7's ladders but for its single-precision fit's noise. A multi-use
object goes in as `BuildMUObject` puts it: its model once, in its own space and its first placement's sector, and a
reference per placement, of its transform and the sector naming its first mesh (`LevelBuildModels`). A model flagged
`eoUsingLOD` gets its impostor first, as `BuildObjectLOD` makes one (`LevelBuildImpostors`): a material of `level_lods`
under `details\lod`, eight quads standing through the model's box with their cells of its strip, the strips read from
the textures root and merged with their normals into `level_lods.dds` and `level_lods_nm.dds` by the same packer as the
details' atlas, 512 by 64 up to 2048 a side, DXT5 with mips. It refuses a draft-quality scene, and exports no breakable
yet. Gunslinger's `l01_test` decompiled and exported comes back to its shipped
`build.prj` on all its area, its sky, sun and dynamic lights exact; the decompiled `labx8`'s compiles in GSC's xrLC and
plays. GSC's own `labx8` scene builds a `level.hom` byte for byte as shipped, and a `level.ltx` of the same rectangle and
sector map parts.

`LevelMapConversion` reads `level.ltx` back as the decompile's first step: the map version and game types into the level
options (the respawn points' game types only where the level has no `level.ltx`), each sector's part of the map
(`[sub_level_map]`, by the sector's number), and the map's rectangle as a level-bound shape, `level_bound_NNNN`, whose
box the Build takes over the objects'. The shipped files were saved again after the Build, holding decimals no float
prints (`126.614072`), so a rectangle comes back as the floats it reads as. Sections the Build writes only from the
build options' text, such as a multiplayer level's `[*_anomaly_sets]`, are refused: the scene's main file holds that
text on one line.

`LevelDetailConversion` gives `level.details` back as `detail_object.part`. The file keeps each detail model's geometry
but not the object it came from, its coordinates moved into the level's atlas, so each model is matched against the
detail objects of the object libraries (`LevelDetailLibrary`: every `.object` under a root's `detail\`, compiled as
`EDetail` exports it): the same positions and indices, its coordinates a remap of the object's, turned or not. A match
comes back as the object's reference at the level's scale range and flags. A model no library draws is refused and the
corners planting it plant nothing, since recovering its texture from the atlas is not done; where none matches, there
is no part. The slots keep their heights, corners and densities, their light zeroed as the Build zeroes it before the
detail lighter runs (inference `detailObjects`). With the SDK's objects as the library, all 14 of `mp_darkvalley`'s
models come back.

`LevelBuildDetails` exports the tool back as `EDetailManager::Export` does: `build.details` of the planted objects
alone, each compiled from its library object, its coordinates moved into the atlas and its texture named
`build_details`, the slots renumbered with their light zeroed; and `build_details.dds`, the objects' textures merged
by `LevelBuildAtlas` (the editor's packer: largest first at the first free spot, turned where only that fits, 256 to
1024 a side) and saved as the editor's `tfADXT1`, DXT1 with mipmaps whose cut-out pixels stay transparent
(`DdsEncoding::with_cut_out`). `LevelBuildExporter::export` takes `LevelBuildSources`: objects roots,
each object from the first holding it, the game materials, and the textures root a scene of details needs.
`mp_darkvalley` decompiled and built gives the shipped `level.details`' header, its 14 objects in order with the same
geometry, shader, flags and scale range, and all 39,601 slots, the light zeroed; only the atlas is its own.

`LevelAiMapConversion` gives `level.ai` back as `ai_map.part`: each node's cell, counted from the world origin, its
height packed over a box of the map's floor and height, its plane, and its links, in the level's order; the covers
are left for xrAI to compute again, and the generator's settings but the patch size take the editor's defaults
(inference `aiMapSettings`). Vanilla `labx8` against GSC's own part: all 6,781 nodes at the same cells, heights
within a millimetre, every link the same, 6,353 planes the same. Its `export` writes `build.aimap` as
`ESceneAIMapTool::Export` does, each node packed again over the box around every node, which `LevelBuildExporter`
adds beside `build.prj`.

## Compile an object into an OGF

`ObjectCompiler::compile` writes an object's OGF and reports what the export made of it (`ObjectCompile`: its `bytes`
and counts; `compile_detail` fills the same for a `.dm`).

As the editor's `PrepareOGF` does, an object with bones exports as a skeleton and any other as a static visual, whatever
its dynamic flag says.

### Skeleton geometry

`ObjectCompiler::build_skeleton_geometry` builds a skeleton object's children as the Actor Editor's `Export OGF`
prepares them:

1. **Corners.** Each face corner takes:
   - the normal the mesh stores for it (`EditableMesh::corner_normals`, written by a decompile or by `blender-xray`),
     else the one its smoothing gives (`EditableMeshNormals`). A decompiled model so keeps its exporter's normals,
     including Blender's corner-angle weighting, where the Actor Editor would recompute them;
   - the last texture coordinate its vertex map reference names;
   - its four strongest bone weights, normalised.
2. **Splits.** Faces go to one split per engine shader, texture and breakable part; a two-sided face is mirrored again.
3. **Packing.** Corners pack into vertices as the export packs them: positions and normals within a millimetre, texture
   coordinates within `EPS_S`, the same bones and weights, looked up through the export's grid over the object's box. A
   near corner moves onto the vertex it stood near. Degenerate and repeated faces are dropped.
4. **Links.** A split's link count is its widest vertex's; a corner of a face following one bone is padded with a
   weightless copy.
5. **Tangents.** The export's tangent pass (NVIDIA's MeshMender, as the SDK calls it) splits a vertex where surrounding
   faces map the texture in disagreeing directions. Vertices are then ordered by first use, and texture coordinates
   moved by whole tiles toward the origin. `ObjectCompileOptions` can request MikkTSpace instead (`TangentMethod`,
   `bevy_mikktspace`), the basis `blender-xray` exports and normal-map bakers assume, split and ordered the same way.
6. **Limits.** A split past 65,535 vertices is cut, faces in order, into children that 16-bit indices can address
   (`SplitPartition`), where the export would overflow. High-poly models from other tools spread a surface over several
   children.

Differences from the editor, none visible in game:

- Children follow the surfaces' order in the object, where the export follows their memory addresses.
- Where the export's tangent pass divides by zero and stores infinities, the compile builds a basis from the normal.

### The OGF written

`ObjectCompiler::compile` writes, as the export does:

- the header: `MT_SKELETON_ANIM` for an object referencing motion libraries, else `MT_SKELETON_RIGID`;
- the description, and one child per split;
- each bone's name, parent and box, and its physics with joint limits as the physics engine reads them;
- the class script, the LODs and the motion references.

Two deliberate departures:

- **Bone boxes** (`BoneBoxes`) are the smallest box flush to a face of the convex hull (`FlushBox`, over the crate's own
  quickhull `ConvexHull`) around every vertex linked to the bone, in rest space. The export fits Wild Magic's `MinBox3`
  around the vertices whose lowest bone it is. This gives one answer per input, is smaller than vanilla's in sum, and
  far fewer vertices leave it as the model animates, so culling bounds and decals on bodies hold better. Bullet hits do
  not use these boxes; they test the bones' physics shapes.
- **Triangle order** uses Forsyth's linear-speed vertex cache optimisation (`VertexCacheOrder`), with vertices numbered
  by first use. The export used D3DX's closed optimiser. The result has as few cache misses as vanilla's order, but not
  its bytes.

### Static objects

A static object is built as `CExportObjectOGF` builds it (`compile/rigid/`):

- one child per surface (not per texture and shader), packed over the static export's grid of the object's box;
- a face dropped only when degenerate;
- each corner's first texture coordinate;
- the same tangent pass, which splits and orders vertices even though the OGF stores no tangent;
- one child is written as the visual itself (`MT_NORMAL`); several as a hierarchy (`MT_HIERRARHY`) carrying the
  description, followed by the collision mesh the export adds;
- a progressive object is written at full detail, without sliding windows.

Two of the export's mistakes are not reproduced:

- A two-sided surface's faces are written once; the export's second walk writes them all again.
- A surface past 65,535 vertices is cut after the tangent pass; the export starts a new part mid-surface and loses a
  face.

### Motions

An object's own motions are written as `ExportMotions` writes them:

- every bone posed at each frame from its channels;
- rotation quantised to 16 bits, translation to 8 or 16 bits within its range, or held where it never changes;
- then how each motion plays and the bone parts it plays on.

A motion that moves the whole object through its root bone is refused for now.

## Verify a compile

`ObjectVerifier::verify` compiles an object and compares the result with the OGF it should replace, by what the engine
reads.

Compared:

- model type, user data, LODs and motion references;
- each bone's name, parent, order and physics;
- the object's own motions by name and length, with their parameters;
- geometry: what each texture and shader draws across all children using them, skinned or static. Vertices by
  position, normal, texture coordinate and weights; distinct triangles by their corners' positions in winding order (a
  face drawn twice counts once; its mirror counts separately).

Not compared: the orders the compile chooses, how a surface's faces fall among children (the export splits by breakable
part; other tools spread one surface over several), tangents, bone boxes, and authoring stamps.

`ObjectVerification` lists the differences in each area and counts what matched. It distinguishes a vertex that differs
only in its normal from one that differs more.

## Checks

Run `cargo test --locked -p xrf-rawdata` from the repository root.

Sources: the [decompiler](src/object/decompile/), the [compiler](src/object/compile/), the
[verifier](src/object/verify/), the [level conversions](src/level/) and the [provenance sidecars](src/provenance/).
