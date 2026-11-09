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

| Conversion                        | Files                                 | Round trip                                                    |
| --------------------------------- | ------------------------------------- | ------------------------------------------------------------- |
| `LevelParticleConversion`         | `level.ps_static` ↔ `ps.part`         | All 18 of vanilla CoP's files, byte for byte                  |
| `LevelSoundSourceConversion`      | `level.snd_static` ↔ `sound_src.part` | 91 files (1,303 sounds) from memory; all but one through text |
| `LevelSoundEnvironmentConversion` | `level.snd_env` ↔ `sound_env.part`    | 15 of 20 byte for byte; 5 within 6.1e-5                       |
| `LevelFogVolumeConversion`        | `level.fog_vol` ↔ `fog_volume.part`   | All 174 files through text                                    |
| `LevelWallmarkConversion`         | `level.wallmarks` ↔ `wallmark.part`   | All 74 files through the binary part; `labx8`'s part exact    |
| `LevelWayConversion`              | `level.game` paths ↔ `way.part`       | 202 files from memory; all but two through text               |
| `LevelSpawnPointConversion`       | `level.game` respawn points, `level.env_mod` ↔ `spawn.part` | 200 of 202 levels through text |

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
  SGM 2.2's `l04u_labx18` an older `env_mod` that keeps no flags. Entity spawns, `level.spawn`, are not converted yet.

### A whole scene

`LevelSceneDecompiler::decompile` reads every pass-through file a level's directory holds and decompiles each through
one context into a scene (`LevelSceneDecompile`), with no geometry yet:

- a part per tool, sorted by name; the spawn part from `level.game` and `level.env_mod` together;
- a main file of a new level named after `LevelDecompileOptions::name`, serving its respawn points' game modes (single
  player where it has none), the editor's build parameters with a weld distance of 0, the camera at the origin;
- a scene GUID from the SHA-256 over each source's name, length and bytes, so the same level always gives the same
  scene.

`write_to_path` writes the sidecar, `<name>.level.xrf.json` (`LevelSceneProvenance`), then the scene, so a failed write
never leaves a decompile that looks authored. The sidecar holds the front-end the options name, each source's SHA-256,
each written file and its sources, and what the decompile chose rather than read (`LevelSceneInference`).

`LevelSceneExporter::export` goes back: the files the Build writes from a scene's parts (`BuildGame`,
`BuildWallmarks`), by name. `level.game`, `level.env_mod` and `level.fog_vol` are written for every scene, empty where
it has no such tool; the rest only where their tool exports something. `write_to_directory` writes them, each staged,
saying of each whether it is new, the same or changed. Since the editor's Build moves every file it may write aside
first, it refuses a directory holding one the scene does not build rather than delete it. Whether existing files may be
replaced is the front-end's to ask (`--force`, a confirmation), as `LevelSceneDecompile::get_existing_paths` lets it ask
before a decompile. Vanilla CoP's `labx8`, `zaton` and `mp_atp`,
and CS's `agroprom_underground`, decompile, read back to the same bytes, and export every file byte for byte; so does
GSC's own `labx8` scene.

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
