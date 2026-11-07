# xrf-rawdata

Converts between the game's compiled assets and the SDK's rawdata, the tree the editors author in. It sits beside
the format crates rather than inside one: a conversion reads one format and writes another, and `xrf-ogf` and
`xrf-editable-object` stay independent of each other.

## Decompiling an OGF into an editable object

`ObjectDecompiler::decompile` turns a skeleton OGF into an `xrf-editable-object` `EditableObjectFile` the Actor Editor
opens. It undoes what the editor's `Export OGF` did:

- every OGF child is a surface (texture and engine shader) and takes only its full-detail sliding window;
- each vertex's links, padded to its child's layout, come back as the bones its point follows and their weights,
  normalised as the export left them, which become one weight map per bone;
- corners split at seams and hard edges are welded back into points, by bit-equal position and the bones followed;
- a face written twice, the copy mirrored with its normals inverted, is one face of a two-sided surface;
- per-face smoothing edge flags (`face_smoth_flags.h`) come back from where welded corners' normals agree;
- bones come from the bones and IK data chunks, joint limits turned back from the physics engine's sense;
- the OGF's own motions come back as the object's, each bone keyed at every frame, its rotation turned back into the
  angles the editor keys, with how each motion plays and the bone parts they play on.

What the OGF does not store is inferred and listed in `ObjectDecompile::inferred`: surface names, compiler shaders and
game materials (the Actor Editor's `default`), the mesh name (`skin`), bone lengths (the SDK's 0.5), smoothing, the
two-sided flag, and for an animated model its motions' curves between frames. `ObjectProvenance` is the sidecar written beside a decompiled file, so it is never taken for an authored
one: the source path and SHA-256, the tool, the time, and every inference.

An OGF that is not a skeleton, or a child with static vertices, is refused for now.

## Compiling an editable object into an OGF

`ObjectCompiler::build_skeleton_geometry` builds a skeleton object's children as the Actor Editor's `Export OGF`
prepares them, so the engine draws what the editor's export would give it:

- every face corner takes its normal from the mesh's smoothing (`EditableMeshNormals`), the last texture coordinate its
  vertex map reference names, and its four strongest bone weights, normalised;
- faces go to one split per engine shader, texture and breakable part, a two-sided face again mirrored;
- corners pack into vertices as the export packs them: positions and normals within a millimetre, texture coordinates
  within `EPS_S`, the same bones and weights, looked up through the export's grid over the object's box, a near corner
  moved onto the vertex it stood near; degenerate and repeated faces are dropped;
- a split's link count is its widest vertex's, a corner of a face that follows one bone padded with a weightless copy;
- tangents and binormals come from the export's tangent pass (NVIDIA's MeshMender as the SDK calls it), which splits a
  vertex where the faces around it map the texture in disagreeing directions, then the vertices are ordered by first
  use and the texture coordinates moved by whole tiles towards the origin.

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

An object's own motions are written as `ExportMotions` writes them: every bone posed at each frame from its channels,
its rotation quantised to 16 bits and its translation to 8 or 16 within its range, or held where it never changes,
then how each motion plays and the bone parts it plays on. A motion moving the whole object through its root bone is
refused for now.

`ObjectVerifier::verify` compiles an object and compares the OGF with one it should stand in for, by what the engine
reads: the model type, user data, LODs and motion references; each bone's name, parent, order and physics; the own
motions by name and length with their parameters; and the geometry, each child paired by texture and shader, its
vertices by position, normal, texture coordinate and weights, and its triangles by their corners' positions. The orders
the compile chooses, tangents, bone boxes and authoring stamps are left out. `ObjectVerification` lists the differences
of each area and counts what matched, telling a vertex that differs in its normal alone from one that differs more.

## Checks

Run `cargo test --locked -p xrf-rawdata` from the repository root. See the [decompiler](src/object/decompile/), the
[compiler](src/object/compile/), the [verifier](src/object/verify/) and the [provenance sidecar](src/provenance/).
