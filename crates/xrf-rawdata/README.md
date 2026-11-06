# xrf-rawdata

Converts between the game's compiled assets and the SDK's rawdata, the tree the editors author in. It sits beside
the format crates rather than inside one: a conversion reads one format and writes another, and `xrf-ogf` and
`xrf-editable-object` stay independent of each other.

## Decompiling an OGF into an editable object

`ObjectDecompiler::decompile` turns a rigid skeleton OGF, every vertex owned by one bone as the corpus's props and
physics objects are, into an `xrf-editable-object` `EditableObjectFile` the Actor Editor opens. It undoes what the
editor's `Export OGF` did:

- every OGF child is a surface (texture and engine shader) and takes only its full-detail sliding window;
- corners split at seams and hard edges are welded back into points, by bit-equal position and bone;
- a face written twice, the copy mirrored with its normals inverted, is one face of a two-sided surface;
- per-face smoothing edge flags (`face_smoth_flags.h`) come back from where welded corners' normals agree;
- bones come from the bones and IK data chunks, joint limits turned back from the physics engine's sense.

What the OGF does not store is inferred and listed in `ObjectDecompile::inferred`: surface names, compiler shaders and
game materials (the Actor Editor's `default`), the mesh name (`skin`), bone lengths (the SDK's 0.5), smoothing and the
two-sided flag. `ObjectProvenance` is the sidecar written beside a decompiled file, so it is never taken for an authored
one: the source path and SHA-256, the tool, the time, and every inference.

An OGF with more than one bone link on a vertex, or with no skinned vertices, is refused for now.

## Checks

Run `cargo test --locked -p xrf-rawdata` from the repository root. See the [decompiler](src/object/decompile/) and the
[provenance sidecar](src/provenance/).
