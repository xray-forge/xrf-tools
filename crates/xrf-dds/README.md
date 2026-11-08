# xrf-dds

Reads, writes, encodes and compares DDS textures, and says which renderer path can load each one.

It is the workspace's only DDS reader: texture building, sprite sheets, the CLI's `dds` commands, the desktop app's
texture views and the renderer's texture upload all go through it. For texture recipes and sprite sheets, use
[xrf-texture](../xrf-texture/README.md).

## Inspect a texture

```rust,no_run
use xrf_dds::DdsFile;

fn main() -> xrf_error::XrfResult {
  let metadata = DdsFile::read_metadata_from_path("gamedata/textures/example.dds")?;

  println!(
    "{} x {}: {}",
    metadata.width,
    metadata.height,
    metadata.get_format_label()
  );

  let texture = DdsFile::read_from_path("gamedata/textures/example.dds")?;
  let pixels = texture.decode_rgba(0)?;

  assert_eq!(pixels.dimensions(), (metadata.width, metadata.height));

  Ok(())
}
```

These are three different amounts of work:

| Call                   | Work                                                                       |
| ---------------------- | -------------------------------------------------------------------------- |
| `read_metadata_from_*` | Reads at most the 148-byte header; the payload size comes from file length |
| `read_from_*`          | Loads the whole payload                                                    |
| `decode_rgba(level)`   | Decodes one mip level to RGBA                                              |

A successful metadata read does not validate the texture.

## Choose an operation

- **Read:** `DdsFile::read_from_path`, `read_from_bytes`, or `read_from_vec`, which keeps the bytes it is given instead
  of copying them.
- **Decode:** `decode_rgba` for one mip level, `to_png` for a PNG of the base level. `get_data` is the payload as
  stored, each layer's chain largest level first.
- **Write:** `write_to_path` or `write_to_bytes`.
- **Encode:** build a `DdsMipChain`, then call `DdsEncoding::encode`, or `encode_cube` with six square chains
  (`+x -x +y -y +z -z`).
- **Check support:** `renderer_support` with a `DdsRenderer`, or `is_xray_compatible` for D3D9, the narrowest path.

About the metadata:

- `DdsMetadata` reports the format as `DdsFormat::D3d`, `Dxgi` or `Unknown`.
- `get_format_label` shows an unrecognised format's FourCC rather than passing it off as supported.
- `get_level_within(side)` names the largest level whose longer side is at most `side` texels, or the smallest level
  when none fits. Use it to measure a coarser level instead of decoding the base.

Decoding a block-compressed level of a flat texture runs in stripes of rows on rayon's pool and gives the same pixels as
one pass.

## Encode pixels and mipmaps

```rust
use xrf_dds::{DdsEncoding, DdsMipChain, DdsMipmaps, ImageFormat, Quality, Rgba, RgbaImage};

fn main() -> xrf_error::XrfResult {
  let pixels = RgbaImage::from_pixel(4, 4, Rgba([255, 0, 0, 255]));
  let chain = DdsMipChain::build(&pixels, DdsMipmaps::Disabled)?;
  let texture = DdsEncoding::new(ImageFormat::BC3RgbaUnorm, Quality::Normal).encode(&chain)?;

  assert_eq!(texture.metadata().mipmap_levels, 1);
  assert_eq!(texture.decode_rgba(0)?.dimensions(), (4, 4));

  Ok(())
}
```

Mip chains:

- `DdsMipmaps::Disabled` writes the base level alone. `DdsMipmaps::Filtered(DdsMipFilter::Box)`, or another kernel,
  writes a full chain down to one pixel. `DdsMipFilter::from_label` matches the SDK's kernel names, ignoring case.
- Every level is reduced from the base, not from the level above.
- Channels are resampled independently and alpha is never premultiplied, because a bump pair packs unrelated planes
  into the four channels.
- The filters implement the published kernels. They are not bit-exact with the closed NVIDIA library the X-Ray
  converter used.

Encoding:

- It consumes the prepared chain without building another, so one chain can be reused to compare candidates.
- Sides need not be multiples of `DdsEncoding::BLOCK_SIZE`; the encoder pads whole blocks and records the real size.
- Block compression is lossy. Compare decoded output with a tolerance.
- Reducing and encoding both run on rayon's pool and give the same bytes however the work is split.

## Compare encodings

`DdsEncodeCandidate` lists the five formats worth offering (BC1, BC2, BC3, RGBA8, BC7) with their cost and renderer
support.

`DdsEncodeAttempt::measure` encodes a chain as one candidate and reports file and GPU bytes, encode time, and a
`DdsImageDifference` (per-channel and overall RMSE, PSNR) against the chain's base level. The difference is loss on top
of the pixels passed in, not distance from some original, and it ranks settings for that texture only.

## Renderer support

Encoding writes legacy DDS headers for BC1, BC2, BC3 and RGBA8, which the D3D9 loader requires, and a DX10 header for
BC7.

`DdsFormatSupport` answers `Supported`, `Unsupported`, or `Unverified` (nobody has checked that loader):

| Path   | Loads                                                         | Refuses or unverified           |
| ------ | ------------------------------------------------------------- | ------------------------------- |
| D3D9   | Every legacy D3D9 format; DXGI legacy block and plain layouts | Refuses BC4, BC5 and BC7        |
| DX11   | Every DXGI block-compressed and plain layout; the DXT family  | Other legacy layouts unverified |
| OpenGL | —                                                             | Everything unverified           |

## Errors

Read, decode and encode return `XrfResult`.

- Decoding fails for a layout neither decoder reads, a cubemap or volume, or a missing mip level.
- Building a chain fails for an image with no pixels.
- `encode_cube` fails unless there are six square faces of equal size and level count.

A header can be readable even when the payload cannot be decoded.

## Checks

Run `cargo test --locked -p xrf-dds` from the repository root.

Sources: the [file API](src/file/dds_file.rs), [metadata](src/file/dds_metadata.rs),
[encoder](src/encode/dds_encoding.rs), [mip chain](src/mip/dds_mip_chain.rs), and
[renderer support](src/renderer/dds_renderer.rs).
