# xrf-texture

X-Ray texture recipes: the sprite sheets the interface is drawn from, the bump pair a surface binds, and the operations
that build each of them. It sits above [xrf-dds](../xrf-dds/README.md), which reads and encodes the DDS files, and
below the CLI's `dds` and `sprite` commands and the desktop app's texture and sprite editors.

## Choose an operation

- `BuildTextureProcessor::build` rebuilds a texture from a source image the way its `.thm` descriptor says, as the
  SDK's `DXTCompress` does; `BuildTextureRecipe::of` reads that descriptor as a recipe.
- `GenerateBumpProcessor::generate` produces a bump pair from a height map and gloss; a bump descriptor's build does
  the same.
- `CropTextureProcessor::crop` extracts a DDS region, optionally fits it, and writes PNG or DDS output.
- `DescriptionSpritePacker::pack_xml_descriptions` and `DescriptionSpriteUnpacker::unpack_xml_descriptions` handle
  sprite sheets an XML description lays out; both take `PackDescriptionOptions`.
- `EquipmentSpritePacker::pack_sprites` and `EquipmentSpriteUnpacker::unpack_sprites` handle the inventory icon sheet.
- `EquipmentGridVerifier::find_overlaps` checks occupied inventory rectangles in resolved LTX data.

`TextureEncodingFormat` and `TextureEncodingQuality` are the five formats and three efforts a texture or a capture is
offered to be written in, as the desktop app's editors ask; `to_candidate` and `to_quality` turn them into
[xrf-dds](../xrf-dds/README.md)'s encoder inputs.

Each operation takes an options value carrying its inputs, output paths, and policy. Resolve game configs with the
intended LTX dialect before passing them to equipment operations. `ImageFileReader::read_rgba` reads authoring input
such as a height map or gloss mask as BMP, DDS, JPEG, PNG or TGA, chosen by its extension; the crate builds no other
`image` codec.

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

  println!("{:?} {:?}, {:?}, omitting {:?}", recipe.layout, recipe.format, recipe.mipmaps, recipe.omissions);

  let built = BuildTextureProcessor::build(&BuildTextureOptions {
    destination: "output/example.dds".into(),
    source: RgbaImage::from_pixel(64, 64, Rgba([128, 128, 255, 255])),
    descriptor,
    normal_map: None,
    quality: Quality::Normal,
  })?;

  println!("{} levels written to {}", built.mipmap_levels, built.destination.display());

  Ok(())
}
```

The descriptor decides everything: the layout, whether there is a mip chain and who reduces it, and whether the
source's alpha means anything; nothing is taken from the file being replaced. DXT1, DXT3, DXT5 and RGBA map to BC1,
BC2, BC3 and uncompressed RGBA. Other formats are refused rather than substituted, DXT1 Alpha included because the
available BC1 encoder does not preserve its one-bit cutout, and so is a descriptor without texture parameters.

| Descriptor                    | Built as                                                                         |
| ----------------------------- | -------------------------------------------------------------------------------- |
| Image, normal map, terrain    | One texture (`BuildTextureLayout::Planar`)                                       |
| Cube map                      | A cube from six square faces side by side, `+x -x +y -y +z -z` (`CubeStrip`)     |
| Bump map                      | The `<name>.dds` and `<name>#.dds` pair, as `DXTCompressBump` builds (`BumpPair`) |
| `Advanced` mip filter         | The SDK's own chain (`BuildTextureAdvancedChain`), fade included                 |
| Any other filter, mip maps on | A chain the filter reduces from the base (`BuildTextureMipmaps::Filtered`)       |

A bump descriptor builds its pair from one source holding the height in its colour and the gloss in its alpha, at the
descriptor's virtual height, DXT5 with a box chain whatever the descriptor's own format and mip settings say, since
`DXTCompressBump` reads neither. The companion's path and the gloss verdict are in `BuildTextureResult::bump`. A bump
descriptor naming an external normal map needs `BuildTextureOptions::normal_map`, the map's pixels and its own
descriptor, which only the caller can find; like the SDK the build refuses a map that is not a 32-bit RGBA normal map of
the source's size, and refuses a normal map for a descriptor that names none.

The `Advanced` chain is `Build32MipLevel` ported: each level the 2x2 average of the one above, faded towards
`fade_color` by `fade_amount` percent from level `fade_delay` on, alpha raised by an eighth, and stopping when the
shorter side reaches one texel. Like the SDK it pulls the blue channel towards the fade colour's red and the red channel
towards its blue, and it refuses a level with an odd side, which the SDK asserts against.

`omissions` names each authored setting the builder does not apply, all of them ones the SDK hands to the closed nvDXT
library: borders, binary alpha, and a fade over a chain nvDXT reduces. The dithering flags are not omissions: nvDXT
dithers only a conversion to 16 bits, none of the formats written here, and documents even that as disabled. Omissions
are information about a successful but partial build, not failures. The destination's directory must already exist.

## Bump pairs

`GenerateBumpProcessor` ports the SDK's `DXTCompressBump`. For a bump named `<bump>` it writes `<bump>.dds` (normals
and gloss) and `<bump>#.dds` (the compression error and the height); `pair_paths` names both, and `bump_of` names the
`<texture>_bump` a texture binds. The second half records what the first lost to DXT5 compression, so both are encoded
before either is written, and a cancelled run writes neither. `GenerateBumpOptions` takes the height map, its gloss, an
optional external normal map, which must match the height in size, the virtual height, the mip filter and the quality.
A bump pair is distinct from a standalone normal-map image. `GenerateBumpResult::is_gloss_too_dark` reports a mean gloss
below 0.1; the pair is written anyway.

As in the SDK, the normals read the colour average truncated to a byte, and the height the second half stores reads
the red channel, rescaled and floored with `EPS_S`. The gloss is the height's own alpha as the SDK reads it
(`GenerateBumpGloss::HeightAlpha`), or a constant or a mask of its own for a source without one. The SDK always reduces
both halves with a box; here the filter is an option, box by default in the CLI and the app. Where a rescaled height or
an error passes 255 the SDK keeps its low eight bits; these are clamped.

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
use xrf_texture::EquipmentGridVerifier;

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
  let overlaps = EquipmentGridVerifier::find_overlaps(&config);

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
