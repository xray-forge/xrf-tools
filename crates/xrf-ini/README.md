# xrf-ini

Reads and writes INI files: `[section]` headers and `key = value` fields, kept in the order they were read or added.
Nothing else is assumed. Every convention beyond that, from the encoding to padded columns and values spanning lines,
is a field of an `IniFormat` the caller names; the methods without a format use `IniFormat::PLAIN`.

This is not [xrf-ltx](../xrf-ltx/README.md). `xrf-ltx` resolves the game's LTX configs: includes, section parents, the
DLTX dialect and whole config trees. `xrf-ini` reads and writes one file as it stands, so a file a program saved comes
back byte for byte, which is what the SDK's own text files need.

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

| Format                    | Encoding     | Line end | Field layout                      | Section end  | Multi-line quoted values | Repeated key  | Order            |
| ------------------------- | ------------ | -------- | --------------------------------- | ------------ | ------------------------ | ------------- | ---------------- |
| `IniFormat::PLAIN`        | UTF-8        | `\n`     | `key = value`                     | none         | no                       | refused       | as given         |
| `IniFormat::XRAY_SAVE_AS` | Windows-1251 | `\r\n`   | `%8s%-32s = %-32s`, right-trimmed | a line `" "` | yes                      | last value    | `strcmp` by name |

`IniFormat::XRAY_SAVE_AS` is what the engine's and the SDK's `CInifile::save_as` writes (`xrCore/xr_ini.cpp`): the
Level Editor's scenes and text parts are saved this way. It reads as `CInifile::Load` reads: a value may be a
double-quoted string spanning lines, as a spawn object's custom data is, and a key set twice in a section keeps its
last value, as other tools' scenes need (bardak's converter writes a glow's `flags` twice). Each value dropped that
way is named in `IniFile::overridden` and left out of a write. A format is a plain struct, so a caller with other
conventions builds its own.

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

## What is read

A `[name]` line opens a section; a header followed by anything but a `;` comment is refused, since section parents are
an LTX feature. A line with `=` is a field, its key and value trimmed at both ends; a line without one is a key with an
empty value. Blank lines and lines starting with `;` are skipped. Values are kept as written: quotes, and in a format
that has them, the line breaks of a multi-line value, joined with the format's line end. Reads accept `\n` and `\r\n`.

## Errors and checks

Reads refuse, with the line, a field before any section, a field with no key, a key set twice in a section where the
format refuses it, a section declared twice (which `CInifile` treats as fatal too), and a quote still open at the end
of the text. Writes refuse what would not read back as itself: an
empty or bracketed section name, a key that is empty, padded, starts with `;` or `[`, or holds `=` or a line break, a
value padded with whitespace, and a line break in a value outside a quoted string the format reads across lines.

Run `cargo test --locked -p xrf-ini` from the repository root. See the [file](src/ini_file.rs), the
[formats](src/ini_format.rs) and the [tests](src/tests/).
