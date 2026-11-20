# xrf-sprite

X-Ray UI sprite sheets: the inventory icon sheet the equipment sections lay out, and the sheets an XML texture
description lays out. It packs icons into a sheet, cuts a sheet back into icons, and checks a layout.

It sits above [xrf-texture](../xrf-texture/README.md), which reads, fits and writes the images, and below the CLI's
`sprite` commands and the desktop app's sprite editor.

## Choose an operation

| Operation                                                                                              | Does                                                                                             |
| ------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `EquipmentSpritePacker::pack_sprites`, `EquipmentSpriteUnpacker::unpack_sprites`                       | Packs or unpacks the inventory icon sheet                                                        |
| `EquipmentGridVerifier::find_overlaps`                                                                 | Finds overlapping inventory rectangles in resolved LTX                                           |
| `DescriptionSpritePacker::pack_xml_descriptions`, `DescriptionSpriteUnpacker::unpack_xml_descriptions` | Packs or unpacks sprite sheets an XML description lays out; both take `DescriptionSpriteOptions` |

- Each operation takes an options value carrying its inputs, output paths and policy.
- Resolve game configs with the intended LTX dialect before passing them to the equipment operations.

## Inventory sprites

The inventory sheet is a grid of 50-pixel cells. A section has an icon only when it declares `$inventory_icon = true`
and whole `inv_grid_x`, `inv_grid_y`, `inv_grid_width` and `inv_grid_height` values, with extents of at least one cell.
Grid fields alone are not enough, because variants inherit them from the sections that own the art.

Check the layout before packing:

```rust
use xrf_ltx::Ltx;
use xrf_sprite::EquipmentGridVerifier;

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

**Unpacking** writes one `<section>.dds` per declared icon and skips an icon outside the sheet. It ignores
`$inventory_icon_path`.

## XML description sheets

XML description sheets use texture and sprite descriptors instead of inventory sections. With `is_strict` they fail
where they would otherwise warn and skip: an unreadable sheet or sprite, or a sprite outside its sheet.

## Output names

Description packing and unpacking and equipment unpacking name files after content (sprite IDs, section names). They
check every name before the first write, and write below their output through their options' `link_policy`, so a
refused name leaves no partial output.

## Errors

These are errors: mismatched sprite sizes, refused names, and read or write failures. Check results too: skipped counts
and cancelled outcomes describe runs that returned `Ok`.

The optional `typescript-bindings` feature derives frontend type metadata.

## Checks

Run `cargo test --locked -p xrf-sprite` from the repository root.

Sources: the [public operations](src/lib.rs) and their options in [description](src/description) and
[equipment](src/equipment).
