# xrf-dds

Reads, decodes, encodes, and compares DDS textures, including mipmaps and X-Ray renderer compatibility.

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

Metadata inspection reads the header. Full reading loads the payload; decoding produces RGBA pixels for the requested
mip level. These are different amounts of work and provide different validation guarantees.

## Choose an operation

- Read a DDS with `DdsFile::read_from_path` or `read_from_bytes`; use `decode_rgba` for one mip level or `to_png` for
  a preview.
- Use `read_metadata_from_path` for header facts without loading pixel data. A successful metadata read does not
  validate the full texture.
- Build a `DdsMipChain`, then pass it to `DdsEncoding::encode`. Reuse the chain when comparing encoding candidates.
- Ask `renderer_support` with a `DdsRenderer` for the intended rendering path.

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

Use `DdsMipmaps::Filtered(DdsMipFilter::Box)` or another supported kernel for a full chain. Encoding consumes the
prepared levels without generating a second chain. Block compression can change pixel values, so compare decoded
output with a tolerance rather than assuming lossless equality.

`DdsEncodeCandidate`, `DdsEncodeAttempt`, and `DdsImageDifference` support evaluating candidate encodings against the
source. Their results describe the tested texture and settings, not a universal quality ranking.

## Compatibility and limitations

Encoding uses legacy DDS headers where the format permits them; formats such as BC7 require a DX10 header.
Compatibility depends on both the format and the renderer.

The mip filters implement their mathematical kernels; they do not promise bit-exact output from the closed NVIDIA
library used by the original X-Ray converter. Header metadata may be readable even when the payload cannot be decoded.
Keep unknown format labels visible when reporting metadata rather than treating them as a supported encoding.

## Errors and checks

Read, decode, and encode methods return `XrfResult`. Unsupported pixel layouts, invalid mip data, and encoder failures
are fallible operations. Run `cargo test --locked -p xrf-dds` from the repository root.

See the [file API](src/file/dds_file.rs) and [encoder](src/encode/dds_encoding.rs). For sprite sheets and texture
recipes, use [xrf-texture](../xrf-texture/README.md).
