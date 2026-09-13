# xrf-texture

Builds X-Ray textures and bump pairs, and packs or extracts UI sprite sheets described by XML or inventory LTX data.

## Choose an operation

- `BuildTextureProcessor::build` uses a texture recipe; `BuildTextureRecipe::of` derives one from a THM descriptor.
- `GenerateBumpProcessor::generate` produces a bump pair from its input and options.
- `CropTextureProcessor::crop` extracts a DDS region, optionally fits it, and writes PNG or DDS output.
- `PackDescriptionProcessor` and `UnpackDescriptionProcessor` handle XML-described sprite sheets.
- `PackEquipmentProcessor::pack_sprites` and `UnpackEquipmentProcessor::unpack_sprites` handle inventory icons.
- `VerifyEquipmentGridProcessor::find_overlaps` checks occupied inventory rectangles in resolved LTX data.

Use the corresponding options type to supply inputs, output paths, and operation policy. Resolve game configs with
the intended LTX dialect before passing them to equipment operations.

## Build from a THM recipe

```rust,no_run
use xrf_db::{ThmFile, XRayByteOrder};
use xrf_dds::{DdsFile, Quality};
use xrf_texture::{BuildTextureOptions, BuildTextureProcessor, BuildTextureRecipe};

fn main() -> xrf_error::XrfResult {
  let descriptor = ThmFile::read_from_path::<XRayByteOrder, _>(&"gamedata/textures/example.thm")?;
  let recipe = BuildTextureRecipe::of(&descriptor)?;

  println!("Format: {:?}; omissions: {:?}", recipe.format, recipe.omissions);

  let source = DdsFile::read_from_path("gamedata/textures/example.dds")?.decode_rgba(0)?;

  BuildTextureProcessor::build(&BuildTextureOptions {
    destination: "output/example.dds".into(),
    source,
    descriptor,
    quality: Quality::Normal,
  })?;

  Ok(())
}
```

Inspect the recipe before writing. Supported THM formats map to BC1, BC2, BC3, or uncompressed RGBA. Unsupported formats
are refused rather than substituted; DXT1 Alpha is refused because the available BC1 encoder does not preserve its
one-bit cutout. Recipe omissions name authored settings the builder does not apply.

## Inventory sprites

Equipment operations use resolved inventory sections and their grid rectangles. Check the layout before packing:

```rust
use xrf_ltx::Ltx;
use xrf_texture::VerifyEquipmentGridProcessor;

fn main() -> xrf_error::XrfResult {
  let config =
    Ltx::read_from_str("[item_a]\ninv_grid_x = 0\ninv_grid_y = 0\ninv_grid_width = 1\ninv_grid_height = 1\n")?;
  assert!(VerifyEquipmentGridProcessor::find_overlaps(&config).is_empty());
  Ok(())
}
```

Grid overlap detection checks rectangles, not whether the icon pixels depict the right item. Packing and unpacking use
their own options to select sources, destinations, and missing-sprite policy. XML description operations use texture
and sprite descriptors instead of inventory sections.

## Bumps and cropping

`GenerateBumpProcessor` generates the X-Ray bump pair, and `pair_paths` identifies its two output paths. Its options
control the source, gloss, and generation settings; a bump pair is distinct from a standalone normal-map image.

Cropping reads mip level zero from a DDS. `CropTextureOptions` gives the region and optional fit dimensions; both fit
dimensions must be present to request fitting. A `.png` output selects PNG; otherwise the operation writes DDS using
the supplied compression and mipmap choices.

## Errors, features, and checks

Missing texture parameters, unsupported recipes, invalid crop bounds, read failures, and write failures are fallible.
Inspect operation results as well as errors: reported omissions are information about a successful but partial recipe.
The optional `typescript-bindings` feature adds frontend type metadata.

Run `cargo test --locked -p xrf-texture` from the repository root.

For DDS bytes, mipmaps, and format compatibility, use [xrf-dds](../xrf-dds/README.md). For THM dependency resolution,
use [xrf-material](../xrf-material/README.md).

See the [public operations](src/lib.rs) and their options in [build](src/build), [bump](src/bump),
[description](src/description), and [equipment](src/equipment).
