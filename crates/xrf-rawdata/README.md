# xrf-rawdata

Converts between the game's compiled assets and the SDK's rawdata, the tree the editors author in. It sits beside
the format crates rather than inside one: a conversion reads one format and writes another, and `xrf-ogf` and
`xrf-editable-object` stay independent of each other.

## Decompiling an OGF into an editable object

`ObjectDecompiler::decompile` turns a skeleton OGF, or a static one (`MT_NORMAL`, `MT_PROGRESSIVE`, or `MT_HIERRARHY`
holding them), into an `xrf-editable-object` `EditableObjectFile` the Actor Editor opens. It undoes what the editor's
`Export OGF` did:

- every OGF child is a surface (texture and engine shader) and takes only its full-detail sliding window;
- a face the export wrote again on the same points, wound the same way, is dropped: the static export walks a two-sided
  surface's faces twice and writes each face and its mirror again;
- each vertex's links, padded to its child's layout, come back as the bones its point follows and their weights,
  normalised as the export left them, which become one weight map per bone;
- corners split at seams and hard edges are welded back into points, by bit-equal position, the bones followed and
  their weights within rounding, so two points the export's `VPack` moved onto one position stay two;
- a face written twice, the copy mirrored with its normals inverted, is one face of a two-sided surface;
- per-face smoothing edge flags (`face_smoth_flags.h`) come back from where welded corners' normals agree, for the
  SDK, and the corners' normals themselves in `blender-xray`'s corner normals chunk, which the SDK does not read;
- bones come from the bones and IK data chunks, joint limits turned back from the physics engine's sense;
- the OGF's own motions come back as the object's, each bone keyed at every frame, its rotation turned back into the
  angles the editor keys, with how each motion plays and the bone parts they play on.

A static OGF comes back as a static object laid out as the SDK lays one out: object flags `0` (progressive where the
OGF has sliding windows), no bones, one mesh whose UV maps are named `Texture` and which holds no weight map, its
vertices following no bone.

What the OGF does not store is inferred and listed in `ObjectDecompile::inferred`: surface names, compiler shaders and
game materials (the Actor Editor's `default`), the mesh name (`skin`, or `mesh` for a static object), a skeleton's bone
lengths (the SDK's 0.5), smoothing, the two-sided flag, and for an animated model its motions' curves between frames
and, for a motion holding one key a bone more than it declares (`blender-xray`'s, for an action starting between
frames), its length from its keys.
`ObjectProvenance` is the sidecar written beside a decompiled file, so it is never taken for an authored one: the source
path and SHA-256, the tool, the time, every inference, and the tangent method the source was made with (MikkTSpace
where its description's convertor names Blender), which `ObjectCompileOptions::following` compiles it back with.

An OGF of any other model type (trees, LODs, particles) is refused.

## Decompiling a level's visuals

`ObjectDecompiler::decompile_level_geometry` turns visuals of a compiled level into one static object through the same
weld, repeat dropping, two-sided folding and smoothing inference. Each `LevelVisualGeometry` is a visual's texture and
shader, the vertices `xrf-level` decodes from `level.geom` and its full-detail indices (`LevelVisualGeometry::read`,
below), in the space they are stored in, a tree's own until its placement is applied. Visuals of one texture and
shader become one surface, and corners welded by position across visuals, so faces that met in the source meet again.

A level stores a normal a byte a component, `(n + 1) * 127.5` truncated, so a two-sided face's mirror comes back one
step (1/127.5) off the exact negation, and the decompile compares level normals within one step, where it compares an
OGF's float normals within `1e-3`. Over vanilla CoP's 22 levels at full detail, 11,692,886 mirrored corners stand one
step off and 36 at none; corners sharing a position agree exactly or differ by over 20 steps but for 5,372 one step
apart.

`LevelVisualGeometry::read` resolves a visual for the decompile: its texture and shader from its own texture chunk or
its shader table row, its vertices, and its full-detail indices through `xrf-level`'s `read_full_detail_indices`, which
takes a static progressive visual's own first window and a progressive tree's from the table of `level.geom` it names.

## Compiling an editable object into an OGF

`ObjectCompiler::compile` writes an object's OGF and reports what the export made of it (`ObjectCompile`). As the
editor's `PrepareOGF` does, an object with bones exports as a skeleton and any other as a static visual, whatever its
dynamic flag says.

`ObjectCompiler::build_skeleton_geometry` builds a skeleton object's children as the Actor Editor's `Export OGF`
prepares them, so the engine draws what the editor's export would give it:

- every face corner takes the normal the mesh stores for it (`EditableMesh::corner_normals`, which a decompile and
  `blender-xray` write), else its smoothing's (`EditableMeshNormals`), the last texture coordinate its vertex map
  reference names, and its four strongest bone weights, normalised: so a decompiled model keeps the normals its
  exporter gave it, Blender's corner-angle weighting included, where the Actor Editor recomputes them from smoothing;
- faces go to one split per engine shader, texture and breakable part, a two-sided face again mirrored;
- corners pack into vertices as the export packs them: positions and normals within a millimetre, texture coordinates
  within `EPS_S`, the same bones and weights, looked up through the export's grid over the object's box, a near corner
  moved onto the vertex it stood near; degenerate and repeated faces are dropped;
- a split's link count is its widest vertex's, a corner of a face that follows one bone padded with a weightless copy;
- tangents and binormals come from the export's tangent pass (NVIDIA's MeshMender as the SDK calls it), which splits a
  vertex where the faces around it map the texture in disagreeing directions, then the vertices are ordered by first
  use and the texture coordinates moved by whole tiles towards the origin; `ObjectCompileOptions` can ask for
  MikkTSpace instead (`TangentMethod`, `bevy_mikktspace`), the basis `blender-xray` exports and normal map bakers
  assume, split and ordered the same way;
- a split past 65,535 vertices is cut, its faces in order, into children 16-bit indices reach (`SplitPartition`),
  where the export would overflow them: high-poly models other tools exported spread a surface over several children.

The order of the children follows the surfaces' order in the object, where the export follows their addresses in
memory, so it can differ; the engine draws the same either way. Where the export's tangent pass divides by a zero
length and stores infinities, the compile builds a basis from the normal instead.

`ObjectCompiler::compile` writes the OGF from those children as the export writes it: the header (`MT_SKELETON_ANIM`
for an object referencing motion libraries, else `MT_SKELETON_RIGID`), the description, a child per split, each bone's
name, parent and box, its physics with the joint limits stored as the physics engine reads them, the class script, the
LODs and the motion references. A bone's box (`BoneBoxes`) is the smallest box flush to a face of the convex hull
(`FlushBox`, over our own quickhull, `ConvexHull`) around every vertex linked to the bone, in its rest space, where the
export fits Wild Magic's `MinBox3` around the vertices whose lowest bone it is: one answer for one input, smaller than
vanilla's in sum, and far fewer vertices leave it as the model animates, so the culling bounds and decals on bodies
hold better. Bullet hits do not read these boxes: they test the bones' physics shapes. Each child's triangles
are ordered for the vertex cache (Forsyth's linear-speed optimisation, `VertexCacheOrder`) and its vertices numbered by
first use, where the export took D3DX's closed optimiser: as few cache misses as vanilla's order, not its bytes.

A static object is built as `CExportObjectOGF` builds it (`compile/rigid/`): a child per surface rather than per texture
and shader, packed over the static export's grid of the object's box, a face dropped only where degenerate, the corners'
first texture coordinate, and the same tangent pass, which splits and orders the vertices though the OGF stores no
tangent. One child is written as the visual itself (`MT_NORMAL`), several as a hierarchy (`MT_HIERRARHY`) carrying the
description, then the collision mesh the export adds. Two of the export's slips are not followed: a two-sided surface's
faces are written once, where its second walk writes them all again, and a surface past 65,535 vertices is cut after
the tangent pass, where the export starts a new part mid-surface and loses a face. A progressive object is written at
full detail, without sliding windows.

An object's own motions are written as `ExportMotions` writes them: every bone posed at each frame from its channels,
its rotation quantised to 16 bits and its translation to 8 or 16 within its range, or held where it never changes,
then how each motion plays and the bone parts it plays on. A motion moving the whole object through its root bone is
refused for now.

`ObjectVerifier::verify` compiles an object and compares the OGF with one it should stand in for, by what the engine
reads: the model type, user data, LODs and motion references; each bone's name, parent, order and physics; the own
motions by name and length with their parameters; and the geometry, what each texture and shader draws over all the
children drawing with them, skinned or static, its vertices by position, normal, texture coordinate and weights, and its
distinct triangles by their corners' positions in winding order, a face drawn twice counted once and its mirror apart. The orders the compile chooses, how a surface's faces fall among children (the export splits one by
breakable part, other tools spread one over several), tangents, bone boxes and authoring stamps are left out. `ObjectVerification` lists the differences
of each area and counts what matched, telling a vertex that differs in its normal alone from one that differs more.

## Checks

Run `cargo test --locked -p xrf-rawdata` from the repository root. See the [decompiler](src/object/decompile/), the
[compiler](src/object/compile/), the [verifier](src/object/verify/) and the [provenance sidecar](src/provenance/).
