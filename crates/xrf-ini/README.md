# xrf-ini

Reads and writes INI files: `[section]` headers and `key = value` fields, kept in the order they were read or added.

Nothing else is assumed. Every other convention (the encoding, padded columns, values spanning lines) is a field of an
`IniFormat` the caller names. Methods without a format use `IniFormat::PLAIN`.

Use [xrf-ltx](../xrf-ltx/README.md) for the game's LTX configs: it resolves includes, section parents, the DLTX dialect
and whole config trees. `xrf-ini` handles one file as it stands, so a file a program saved comes back byte for byte, as
the SDK's own text files need.

## Read and write a plain file

```rust
use xrf_ini::IniFile;

fn main() -> xrf_error::XrfResult {
  let mut file = IniFile::read_from_str("[window]\nwidth = 800\n")?;

  assert_eq!(file.section_required("window")?.get_parsed::<u32>("width")?, 800);

  file.section_mut("window").set("height", "600");

  assert_eq!(file.write_to_string()?, "[window]\nwidth = 800\nheight = 600\n");

  Ok(())
}
```

## Formats

| Setting                  | `IniFormat::PLAIN` | `IniFormat::XRAY_SAVE_AS`         |
| ------------------------ | ------------------ | --------------------------------- |
| Encoding                 | UTF-8              | Windows-1251                      |
| Line end                 | `\n`               | `\r\n`                            |
| Field layout             | `key = value`      | `%8s%-32s = %-32s`, right-trimmed |
| Section end              | none               | a line holding `" "`              |
| Multi-line quoted values | no                 | yes                               |
| Repeated key             | refused            | last value wins                   |
| Section order            | as given           | `strcmp` by name                  |

A format is a plain struct, so a caller with other conventions can build its own.

`IniFormat::XRAY_SAVE_AS` matches the engine's and SDK's `CInifile::save_as` (`xrCore/xr_ini.cpp`); the Level Editor
saves its scenes and text parts this way. It reads as `CInifile::Load` does:

- A value may be a double-quoted string spanning lines, as a spawn object's custom data is.
- A key set twice in a section keeps its last value, which other tools' scenes need (bardak's converter writes a glow's
  `flags` twice). Each dropped value is listed in `IniFile::overridden` and left out of a write.

```rust
use xrf_ini::{IniFile, IniFormat};

fn main() -> xrf_error::XrfResult {
  let text = "[guid]\r\n        guid_g0                          = 1\r\n \r\n";
  let file = IniFile::read_from_str_as(text, &IniFormat::XRAY_SAVE_AS)?;

  assert_eq!(file.get("guid", "guid_g0"), Some("1"));
  assert_eq!(file.write_to_string_as(&IniFormat::XRAY_SAVE_AS)?, text);

  Ok(())
}
```

## How lines are read

- `[name]` opens a section. A header followed by anything but a `;` comment is refused; section parents are an LTX
  feature.
- A line with `=` is a field; key and value are trimmed at both ends.
- A line without `=` is a key with an empty value.
- Blank lines and lines starting with `;` are skipped.
- Values are kept as written, quotes included. A multi-line value's line breaks are joined with the format's line end.
- Both `\n` and `\r\n` are accepted.

## Errors

Reads refuse, naming the line:

- a field before any section, or a field with no key;
- a repeated key, where the format refuses it;
- a section declared twice (`CInifile` treats this as fatal too);
- a quote still open at the end of the text.

Writes refuse what would not read back as itself:

- an empty or bracketed section name;
- a key that is empty, padded, starts with `;` or `[`, or holds `=` or a line break;
- a value padded with whitespace;
- a line break in a value, unless inside a quoted string the format reads across lines.

## Checks

Run `cargo test --locked -p xrf-ini` from the repository root.

Sources: the [file](src/ini_file.rs), the [formats](src/ini_format.rs) and the [tests](src/tests/).
