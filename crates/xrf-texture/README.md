# xrf-texture

X-Ray texture recipes: the sprite sheets the interface is drawn from, the bump pair a surface binds, and the operations
that build each of them. It sits above [xrf-dds](../xrf-dds/README.md), which reads and encodes the DDS files, and
below the CLI's `dds` and `sprite` commands and the desktop app's texture and sprite editors.

## Choose an operation

- `BuildTextureProcessor::build` rebuilds a texture from a source image the way its `.thm` descriptor says;
  `BuildTextureRecipe::of` reads that descriptor as a recipe.
- `GenerateBumpProcessor::generate` produces a bump pair from a height map and gloss.
- `CropTextureProcessor::crop` extracts a DDS region, optionally fits it, and writes PNG or DDS output.
- `PackDescriptionProcessor::pack_xml_descriptions` and `UnpackDescriptionProcessor::unpack_xml_descriptions` handle
  sprite sheets an XML description lays out; both take `PackDescriptionOptions`.
- `PackEquipmentProcessor::pack_sprites` and `UnpackEquipmentProcessor::unpack_sprites` handle the inventory icon sheet.
- `VerifyEquipmentGridProcessor::find_overlaps` checks occupied inventory rectangles in resolved LTX data.

Each operation takes an options value carrying its inputs, output paths, and policy. Resolve game configs with the
intended LTX dialect before passing them to equipment operations. `read_image_as_rgba` reads authoring input such as a
height map or gloss mask in any format the `image` decoder recognises.

## Build from a THM recipe

```rust,no_run
use xrf_dds::{Quality, Rgba, RgbaImage};
use xrf_texture::{BuildTextureOptions, BuildTextureProcessor, BuildTextureRecipe};
use xrf_thm::{ThmFile, ThmFormat};

fn main() -> xrf_error::XrfResult {
  // A descriptor as the SDK starts one, asking for DXT5 with a mip chain; a real one is read with `ThmFile`.
  let mut descriptor = ThmFile::new_texture();

  if let Some(parameters) = descriptor.texture_param.as_mut() {
    parameters.format = ThmFormat::Dxt5;
  }

  let recipe = BuildTextureRecipe::of(&descriptor)?;

  println!("{:?}, {:?}, omitting {:?}", recipe.format, recipe.mipmaps, recipe.omissions);

  let built = BuildTextureProcessor::build(&BuildTextureOptions {
    destination: "output/example.dds".into(),
    source: RgbaImage::from_pixel(64, 64, Rgba([128, 128, 255, 255])),
    descriptor,
    quality: Quality::Normal,
  })?;

  println!("{} levels written to {}", built.mipmap_levels, built.destination.display());

  Ok(())
}
```

The descriptor decides everything: the layout, whether there is a mip chain and which kernel reduces it, and whether
the source's alpha means anything; nothing is taken from the file being replaced. DXT1, DXT3, DXT5 and RGBA map to BC1,
BC2, BC3 and uncompressed RGBA. Other formats are refused rather than substituted, DXT1 Alpha included because the
available BC1 encoder does not preserve its one-bit cutout, and so is a descriptor without texture parameters. The
`Advanced` mip filter is reduced with a box kernel. `omissions` names each authored setting the builder does not apply
(the `Advanced` fade, fades, borders, dithering, binary alpha); most shipped descriptors ask for colour dithering, so an
empty list is the exception. Omissions are information about a successful but partial build, not failures. The
destination's directory must already exist.

## Bump pairs

`GenerateBumpProcessor` ports the SDK's `DXTCompressBump`. It writes `<destination>_bump.dds` (normals and gloss) and
`<destination>_bump#.dds` (the compression error and the height); `pair_paths` names both. The second half records what
the first lost to DXT5 compression, so both are encoded before either is written, and a cancelled run writes neither.
`GenerateBumpOptions` takes the height map, a constant or masked gloss, an optional external normal map, which must
match the height in size, the virtual height, the mip filter and the quality. A bump pair is distinct from a standalone
normal-map image. `GenerateBumpResult::is_gloss_too_dark` reports a mean gloss below 0.1; the pair is written anyway.

## Cropping

Cropping reads mip level zero from a DDS. `CropTextureOptions` gives the region and optional fit dimensions; both fit
dimensions must be present to request fitting. An output whose extension is `.png`, in any case, is written as PNG;
anything else is written as DDS with the supplied compression and mipmap choices. A region outside the source is
refused before anything is written.

## Inventory sprites

The inventory sheet is a grid of 50-pixel cells. A section takes part only when it declares `$inventory_icon = true`
and whole `inv_grid_x`, `inv_grid_y`, `inv_grid_width` and `inv_grid_height` values, the extents at least one cell;
grid fields alone do not make a section's icon, because variants inherit them from sections that own the art. Check
the layout before packing:

```rust
use xrf_ltx::Ltx;
use xrf_texture::VerifyEquipmentGridProcessor;

fn main() -> xrf_error::XrfResult {
  let config = Ltx::read_from_str(
    "[wpn_long]\n\
     $inventory_icon = true\n\
     inv_grid_x = 15\n\
     inv_grid_y = 5\n\
     inv_grid_width = 4\n\
     inv_grid_height = 1\n\
     [wpn_short]\n\
     $inventory_icon = true\n\
     inv_grid_x = 16\n\
     inv_grid_y = 5\n\
     inv_grid_width = 2\n\
     inv_grid_height = 1\n",
  )?;
  let overlaps = VerifyEquipmentGridProcessor::find_overlaps(&config);

  assert_eq!(overlaps.len(), 1);
  assert_eq!((overlaps[0].first.as_str(), overlaps[0].second.as_str()), ("wpn_long", "wpn_short"));
  assert_eq!((overlaps[0].cell, overlaps[0].overlapping_cells), ((16, 5), 2));

  Ok(())
}
```

Identical rectangles are not reported, because variants share a slot by design. Overlap detection checks rectangles,
not whether the icon pixels depict the right item.

Packing reads each icon as `<section>.png` or `<section>.dds` from the source directory, or from
`$inventory_icon_path`, fits it into its rectangle, and writes the whole sheet once at the end, so a cancelled run
replaces nothing. An icon it cannot read is skipped with a warning, or fails the run when `is_strict` is set; a slot
claimed by two sections with different art is warned about, and the one packed last wins. Unpacking writes one
`<section>.dds` per declared icon, skips an icon outside the sheet, and does not yet honour `$inventory_icon_path`.
XML description operations use texture and sprite descriptors instead of inventory sections; with `is_strict` they fail
where they would otherwise warn and skip an unreadable sheet or sprite, or a sprite outside its sheet.

Description packing and unpacking and equipment unpacking name files after content (sprite IDs, section names). They
check every name before the first write and write below their output through the `link_policy` of their options, so a
refused name leaves no partial output behind.

## Errors, features, and checks

Unsupported recipes, missing texture parameters, invalid crop bounds, mismatched sprite sizes, refused names, read
failures, and write failures are errors. Inspect operation results as well: omissions, skipped counts, and cancelled
outcomes describe runs that returned `Ok`. The optional `typescript-bindings` feature derives frontend type metadata.

Run `cargo test --locked -p xrf-texture` from the repository root.

For DDS bytes, mipmaps, and format compatibility, use [xrf-dds](../xrf-dds/README.md). For THM dependency resolution,
use [xrf-material](../xrf-material/README.md).

See the [public operations](src/lib.rs) and their options in [build](src/build), [bump](src/bump), [crop](src/crop),
[description](src/description), and [equipment](src/equipment).
