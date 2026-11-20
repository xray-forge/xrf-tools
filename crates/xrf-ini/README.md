# xrf-ini

Reads and writes INI files: `[section]` headers and `key = value` fields, kept in the order they were read or added.

Every other convention (the encoding, padded columns, values spanning lines) is a field of an `IniFormat` the caller
names; methods without an `_as` suffix use `IniFormat::PLAIN`.

`xrf-ini` handles one file as it stands, so a file a program saved comes back byte for byte, as the SDK's own text
files need. For the game's LTX configs, with includes, section parents, the DLTX dialect and whole config trees, use
[xrf-ltx](../xrf-ltx/README.md).

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

`sections()` and `fields()` read a file and a section in order. They change only through `section_mut`,
`insert_section`, `set` and `remove`, which never add a second section of one name or a second field of one key.

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
saves its scenes and text parts this way. A file `save_as` wrote reads as the engine reads it:

- A value may be a double-quoted string spanning lines, as a spawn object's custom data is.
- A key set twice in a section keeps its last value, which other tools' scenes need (bardak's converter writes a glow's
  `flags` twice). Each dropped value is listed in `IniFile::overridden()` and left out of a write.

A file edited by hand can hold things `save_as` never writes. There `xrf-ini` keeps the file as it is rather than
reading it the engine's way:

- In `key = 1 ; note` the value is `1 ; note`; only lines starting with `;` are comments. Use
  [xrf-ltx](../xrf-ltx/README.md) for the engine's comment rules.
- Section names keep their case: `[A]` and `[a]` are two sections, where the engine sees one.
- A line `= value` with no key is an error.

Values are written exactly as set. The engine reformats some values as it saves them, writing a list as `1, 2, 3`;
format a value that way before setting it when the output must match the editor's byte for byte.

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

- `[name]` opens a section. Section parents (`[name]:parent`) are an LTX feature and are refused.
- A line with `=` is a field; key and value are trimmed at both ends.
- A line without `=` is a key with an empty value.
- Blank lines and lines starting with `;` are skipped.
- Values are kept as written, quotes included. A multi-line value's line breaks are joined with the format's line end.
- Both `\n` and `\r\n` are accepted.
- A byte order mark at the start of a file or bytes is skipped; `read_from_str` takes text without one.

## Errors

Reads refuse, naming the line:

- a section header with no `]`, or followed by anything but a `;` comment;
- a field before any section, or a field with no key;
- a repeated key, where the format refuses it;
- a section declared twice (`CInifile` treats this as fatal too);
- a quote still open at the end of the text.

Writes refuse what would not read back as itself:

- an empty or bracketed section name, or a section named twice;
- a key that is empty, padded, starts with `;` or `[`, holds `=` or a line break, or is set twice in its section;
- a value padded with whitespace;
- a line break in a value, unless inside a quoted string the format reads across lines;
- a value with an unclosed quote, where the format would read the next lines into it.

## Checks

Run `cargo test --locked -p xrf-ini` from the repository root.

Sources: the [file](src/ini_file.rs), the [formats](src/ini_format.rs) and the [tests](src/tests/).
