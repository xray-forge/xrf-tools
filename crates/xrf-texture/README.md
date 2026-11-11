# xrf-texture

X-Ray texture recipes and the operations that build them: textures from `.thm` descriptors, bump pairs, UI and
inventory sprite sheets, and texture measurements.

It sits above [xrf-dds](../xrf-dds/README.md), which reads and encodes DDS files, and below the CLI's `dds` and `sprite`
commands and the desktop app's texture and sprite editors. For THM dependency resolution, use
[xrf-material](../xrf-material/README.md).

## Choose an operation

| Operation                                                                                              | Does                                                                                           |
| ------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------- |
| `BuildTextureProcessor::build`                                                                         | Rebuilds a texture from a source image as its `.thm` says (the SDK's `DXTCompress`)            |
| `BuildTextureRecipe::of`                                                                               | Reads a descriptor as a recipe                                                                 |
| `GenerateBumpProcessor::generate`                                                                      | Produces a bump pair from a height map and gloss                                               |
| `CropTextureProcessor::crop`                                                                           | Extracts a DDS region, optionally fits it, writes PNG or DDS                                   |
| `DescriptionSpritePacker::pack_xml_descriptions`, `DescriptionSpriteUnpacker::unpack_xml_descriptions` | Packs or unpacks sprite sheets an XML description lays out; both take `PackDescriptionOptions` |
| `EquipmentSpritePacker::pack_sprites`, `EquipmentSpriteUnpacker::unpack_sprites`                       | Packs or unpacks the inventory icon sheet                                                      |
| `EquipmentGridVerifier::find_overlaps`                                                                 | Finds overlapping inventory rectangles in resolved LTX                                         |
| `TextureAlbedo::read`, `AlbedoVerdict`                                                                 | Measures a diffuse texture's albedo and judges it                                              |
| `BumpAlignment::compare`, `BumpAlignmentVerdict`                                                       | Measures whether a bump belongs to its diffuse, and judges it                                  |

General points:

- Each operation takes an options value carrying its inputs, output paths and policy.
- Resolve game configs with the intended LTX dialect before passing them to the equipment operations.
- `ImageFileReader::read_rgba` reads authoring input (a height map, a gloss mask) as BMP, DDS, JPEG, PNG or TGA, chosen
  by extension. No other `image` codec is built.
- `TextureEncodingFormat` and `TextureEncodingQuality` are the five formats and three effort levels the desktop app's
  editors offer; `to_candidate` and `to_quality` turn them into [xrf-dds](../xrf-dds/README.md)'s encoder inputs.

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
source's alpha means anything. Nothing is taken from the file being replaced. The destination's directory must exist.

**Formats.** DXT1, DXT3, DXT5 and RGBA map to BC1, BC2, BC3 and uncompressed RGBA, and DXT1 Alpha to BC1 whose one
bit of alpha cuts out the texels below half (`DdsEncoding::with_cut_out`). Other formats are refused, not substituted.
A descriptor without texture parameters is refused too.

| Descriptor                    | Built as                                                                          |
| ----------------------------- | --------------------------------------------------------------------------------- |
| Image, normal map, terrain    | One texture (`BuildTextureLayout::Planar`)                                        |
| Cube map                      | A cube from six square faces side by side, `+x -x +y -y +z -z` (`CubeStrip`)      |
| Bump map                      | The `<name>.dds` and `<name>#.dds` pair, as `DXTCompressBump` builds (`BumpPair`) |
| `Advanced` mip filter         | The SDK's own chain (`BuildTextureAdvancedChain`), fade included                  |
| Any other filter, mip maps on | A chain the filter reduces from the base (`BuildTextureMipmaps::Filtered`)        |

### Bump descriptors

- The pair is built from one source holding height in its colour and gloss in its alpha, at the descriptor's virtual
  height.
- It is always DXT5 with a box chain, whatever the descriptor's format and mip settings say, since `DXTCompressBump`
  reads neither.
- The companion's path and the gloss verdict are in `BuildTextureResult::bump`.
- A descriptor naming an external normal map needs `BuildTextureOptions::normal_map` (the map's pixels and its own
  descriptor), which only the caller can find. Like the SDK, the build refuses a map that is not a 32-bit RGBA normal
  map of the source's size, and refuses a normal map for a descriptor that names none.

### The `Advanced` chain

The `Advanced` chain ports `Build32MipLevel`:

- each level is the 2×2 average of the one above;
- from level `fade_delay` on, it fades toward `fade_color` by `fade_amount` percent;
- alpha is raised by an eighth;
- it stops when the shorter side reaches one texel.

Like the SDK, it pulls the blue channel toward the fade colour's red and the red channel toward its blue, and refuses a
level with an odd side, which the SDK asserts against.

### Omissions

`omissions` lists each authored setting the builder does not apply, all of them ones the SDK passes to the closed nvDXT
library: borders, binary alpha, and a fade over a chain nvDXT reduces.

- Dithering flags are not omissions: nvDXT dithers only conversions to 16 bits, none of the formats written here, and
  documents even that as disabled.
- Omissions describe a successful but partial build, not failures.

## Bump pairs

`GenerateBumpProcessor` ports the SDK's `DXTCompressBump`. For a bump named `<bump>` it writes:

- `<bump>.dds`: normals and gloss;
- `<bump>#.dds`: the compression error and the height. It records what the first file lost to DXT5, so both are
  encoded before either is written, and a cancelled run writes neither.

`pair_paths` names both files; `bump_of` names the `<texture>_bump` a texture binds. A bump pair is not the same as a
standalone normal-map image.

`GenerateBumpOptions` takes the height map, its gloss, an optional external normal map (which must match the height in
size), the virtual height, the mip filter and the quality.

Behaviour matching the SDK:

- Normals read the colour average truncated to a byte.
- The stored height reads the red channel, rescaled and floored with `EPS_S`.
- Gloss is the height's own alpha as the SDK reads it (`GenerateBumpGloss::HeightAlpha`), or a constant or separate
  mask for a source without one.
- `GenerateBumpResult::is_gloss_too_dark` reports a mean gloss below 0.1; the pair is written anyway.

Differences from the SDK:

- The SDK always reduces both halves with a box filter; here the filter is an option (box by default in the CLI and the
  app).
- Where a rescaled height or an error exceeds 255, the SDK keeps its low eight bits; these are clamped.

## Albedo

`TextureAlbedo::read` measures how a diffuse texture's colours sit as albedo.

- It decodes the largest level within 512 texels a side, each channel through `xrf_math::srgb_decoded`, as the renderer
  decodes colour textures.
- It measures the texels a surface draws: all of them, or a cut-out's texels its alpha test keeps. `None` means the
  alpha test keeps nothing.

It reports:

- the mean colour;
- the luminance's 5th, 50th and 95th percentiles;
- the share of texels under 0.03 and over 0.9 (`AlbedoVerdict::FLOOR` and `CEILING`: charcoal and fresh snow);
- the coarse spread: the stops between the brighter and darker tenth of an 8×8 grid of cells. Painted large-scale light
  and shade widen it, as does an atlas of unlike materials.

`AlbedoVerdict` judges the median against the range unlit materials span, so a few highlights or a black border do not
decide it.

## Bump alignment

`BumpAlignment::compare` measures whether a bump belongs to its diffuse.

- `BumpAlignmentLevel::read` decodes a diffuse or a bump at its largest level within 512 texels a side, so a survey can
  release each file before the comparison, which takes most of the time.
- The two levels are brought to the coarser of the two along each side (sampled over the same texture coordinates
  whatever their proportions).
- Their edges are correlated texel by texel: the diffuse's colour gradient against the gradient of the bump's normal,
  ignoring gloss. A set authored together has its cracks, seams and atlas outlines at the same texels in both.
- The same correlation with the bump shifted along the tile (the highest of eight shifts placed by Roberts' R2
  sequence) is what any two textures share by chance. The verdict judges the lift over it:

| Verdict       | Lift                                  |
| ------------- | ------------------------------------- |
| `Aligned`     | 0.1 or more                           |
| `Doubtful`    | Between 0.03 and 0.1                  |
| `Mismatched`  | Under 0.03                            |
| `Featureless` | Either texture has nothing to compare |

Edges are compared rather than slopes because an atlas's bump lies flat along the outlines where its diffuse changes
most, and at 256 texels a bump's fine relief averages away; both misread stock Anomaly's aligned atlases.

## Cropping

- Cropping reads mip level zero from a DDS.
- `CropTextureOptions` gives the region and optional fit dimensions; both fit dimensions are needed to fit.
- An output ending in `.png` (any case) is written as PNG; anything else as DDS, with the supplied compression and
  mipmap choices.
- A region outside the source is refused before anything is written.

## Inventory sprites

The inventory sheet is a grid of 50-pixel cells. A section has an icon only when it declares `$inventory_icon = true`
and whole `inv_grid_x`, `inv_grid_y`, `inv_grid_width` and `inv_grid_height` values, with extents of at least one cell.
Grid fields alone are not enough, because variants inherit them from the sections that own the art.

Check the layout before packing:

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

- Identical rectangles are not reported; variants share a slot by design.
- It checks rectangles, not whether the pixels show the right item.

**Packing** reads each icon as `<section>.png` or `<section>.dds` from the source directory (or from
`$inventory_icon_path`), fits it into its rectangle, and writes the whole sheet once at the end, so a cancelled run
replaces nothing.

- An unreadable icon is skipped with a warning, or fails the run when `is_strict` is set.
- A slot claimed by two sections with different art is warned about; the last packed wins.

**Unpacking** writes one `<section>.dds` per declared icon and skips an icon outside the sheet. It does not yet honour
`$inventory_icon_path`.

**XML description sheets** use texture and sprite descriptors instead of inventory sections. With `is_strict` they fail
where they would otherwise warn and skip: an unreadable sheet or sprite, or a sprite outside its sheet.

**Output names.** Description packing and unpacking and equipment unpacking name files after content (sprite IDs,
section names). They check every name before the first write, and write below their output through their options'
`link_policy`, so a refused name leaves no partial output.

## Errors

These are errors: unsupported recipes, missing texture parameters, invalid crop bounds, mismatched sprite sizes,
refused names, and read or write failures.

Check results too: omissions, skipped counts and cancelled outcomes describe runs that returned `Ok`.

The optional `typescript-bindings` feature derives frontend type metadata.

## Checks

Run `cargo test --locked -p xrf-texture` from the repository root.

Sources: the [public operations](src/lib.rs) and their options in [build](src/build), [bump](src/bump),
[crop](src/crop), [description](src/description), and [equipment](src/equipment).
