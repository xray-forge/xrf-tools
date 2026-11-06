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
- bones come from the bones and IK data chunks, joint limits turned back from the physics engine's sense.

What the OGF does not store is inferred and listed in `ObjectDecompile::inferred`: surface names, compiler shaders and
game materials (the Actor Editor's `default`), the mesh name (`skin`), bone lengths (the SDK's 0.5), smoothing and the
two-sided flag. `ObjectProvenance` is the sidecar written beside a decompiled file, so it is never taken for an authored
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
length and stores infinities, the compile builds a basis from the normal instead. The triangle order the engine draws
in, and writing the OGF itself, are not built yet.

## Checks

Run `cargo test --locked -p xrf-rawdata` from the repository root. See the [decompiler](src/object/decompile/), the
[compiler](src/object/compile/) and the [provenance sidecar](src/provenance/).
