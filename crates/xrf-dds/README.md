# xrf-dds

DDS textures as this workspace reads, writes and weighs them: the header facts a surface reports, the pixels behind
them, the X-Ray converter's mip kernels, and which renderer path will load what. Texture building, sprite sheets, the
CLI's `dds` commands, the desktop app's texture views and the renderer's texture upload all decode and encode through
it; there is no second DDS reader.

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

Metadata inspection reads at most the 148-byte header and derives the payload size from the file length. Full reading
loads the payload; decoding produces RGBA pixels for the requested mip level. These are different amounts of work and
provide different validation guarantees: a successful metadata read does not validate the texture.

## Choose an operation

- Read a DDS with `DdsFile::read_from_path` or `read_from_bytes`; header facts alone with `read_metadata_from_path` or
  `read_metadata_from_bytes`.
- Decode with `decode_rgba` for one mip level, or `to_png` for a PNG of the base level. `get_data` is the payload as
  stored, every layer's chain largest level first.
- Write with `write_to_path` or `write_to_bytes`.
- Build a `DdsMipChain`, then pass it to `DdsEncoding::encode`, or six square chains to `encode_cube`
  (`+x -x +y -y +z -z`). Reuse the chain when comparing encoding candidates.
- Ask `renderer_support` with a `DdsRenderer` for one rendering path, or `is_xray_compatible` for the D3D9 path, the
  narrowest of the three.

`DdsMetadata` reports the format as `DdsFormat::D3d`, `Dxgi` or `Unknown`; `get_format_label` keeps an unrecognised
format's FourCC visible rather than treating it as a supported encoding. `get_level_within(side)` names the largest
level whose longer side holds `side` texels at most, the smallest the file carries where none does, for a measure that
reads a coarser level instead of decoding the base.

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

`DdsMipmaps::Disabled` writes the base level alone; `DdsMipmaps::Filtered(DdsMipFilter::Box)` or another kernel writes
a full chain down to one pixel. Every level is reduced from the base rather than from the level above, channels are
resampled independently, and alpha is never premultiplied, because a bump pair packs unrelated planes into the four
channels. `DdsMipFilter::from_label` matches a kernel by the SDK's name, ignoring case. Encoding consumes the prepared
levels without generating a second chain. Block compression can change pixel values, so compare decoded output with a
tolerance rather than assuming lossless equality. Sides need not be multiples of `DdsEncoding::BLOCK_SIZE`; the
encoder pads whole blocks and records the unpadded size.

`DdsEncodeCandidate` lists the five formats worth offering (BC1, BC2, BC3, RGBA8, BC7) with their cost and renderer
support. `DdsEncodeAttempt::measure` encodes a chain as one candidate and reports file and GPU bytes, encode time, and a
`DdsImageDifference` (per-channel and overall RMSE, PSNR) against the chain's base level. That difference is loss on top
of the pixels passed in, not distance from an original, and describes the tested texture and settings rather than a
universal quality ranking.

## Compatibility and limitations

Encoding writes legacy DDS headers for BC1, BC2, BC3 and RGBA8, which the D3D9 loader requires; BC7 is written with a
DX10 header. `DdsFormatSupport` has three answers: `Supported`, `Unsupported`, and `Unverified`, which means nobody has
read that loader. D3D9 loads every legacy D3D9 format and the DXGI legacy block and uncompressed layouts, and refuses
BC4, BC5 and BC7. DX11 loads every block-compressed and uncompressed DXGI layout and the DXT families, and leaves other
legacy layouts unverified. OpenGL answers `Unverified` for everything.

The mip filters implement their published mathematical kernels; they do not promise bit-exact output from the closed
NVIDIA library the original X-Ray converter used. Header metadata may be readable even when the payload cannot be
decoded.

## Errors and checks

Read, decode, and encode methods return `XrfResult`. Decoding fails for a layout neither decoder reads, a cubemap or
volume, or a mip level the file lacks; building a chain fails for an image with no pixels; encoding a cube fails unless
there are six square faces alike in size and levels. Run `cargo test --locked -p xrf-dds` from the repository root.

See the [file API](src/file/dds_file.rs), [metadata](src/file/dds_metadata.rs), [encoder](src/encode/dds_encoding.rs),
[mip chain](src/mip/dds_mip_chain.rs), and [renderer support](src/renderer/dds_renderer.rs). For sprite sheets and
texture recipes, use [xrf-texture](../xrf-texture/README.md).
