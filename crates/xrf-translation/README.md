# xrf-translation

Reads, edits, verifies and builds S.T.A.L.K.E.R. translations, as multilingual JSON sources and per-language XML string
tables.

The game shows text through string tables: XML files under `configs\text\<language>`, each mapping a string ID to one
language's text. XRF authors the same text as JSON sources, one file per table holding every language, and builds the
string tables from them.

This crate owns both formats, the import from game tables into sources, and the read model the desktop app's
translation editor and [xrf-dialog](../xrf-dialog/README.md) resolve text through.

## Source format

A JSON source maps each string ID to language variants. A variant is a string, an array of lines, or `null`:

```json
{
  "greeting": {
    "eng": "Hello, stalker.",
    "ukr": null
  },
  "instructions": {
    "eng": ["Find the guide.", "Return before dark."]
  }
}
```

- An array becomes the engine's literal `\n` line-break sequence on build.
- A missing or `null` translation is not an empty string: the game shows the ID itself.
- Language keys are `TranslationLanguage` codes: `eng`, `fra`, `ger`, `ita`, `pol`, `rus`, `spa`, `ukr`.
- A source file is any `*.json` (any case). Reading one refuses a repeated ID or language.

```rust
use xrf_translation::TranslationVariant;

let lines = TranslationVariant::MultiString(vec![String::from("Find the guide."), String::from("Return before dark.")]);

assert_eq!(lines.to_single_line(), "Find the guide.\\nReturn before dark.");
```

## Code pages

Each language's string table is written in the code page the game draws it with. The engine never decodes a table: the
font maps each byte to a glyph, and the font is chosen by prefix.

| Languages                  | Font prefix | Code page    |
| -------------------------- | ----------- | ------------ |
| `fra`, `ger`, `ita`, `spa` | `_west`     | Windows-1252 |
| `pol`                      | `_cent`     | Windows-1250 |
| `eng`, `rus`, `ukr`        | none        | Windows-1251 |

`TranslationLanguage::get_encoding` returns it. Not every Unicode character can be saved in every language.

## Choose an operation

| Operation                                              | Reads                       | Does                                        |
| ------------------------------------------------------ | --------------------------- | ------------------------------------------- |
| `TranslationBuilder::build_roots`                      | JSON sources through roots  | Compiles game string tables                 |
| `TranslationVerifier::verify_roots`                    | JSON sources through roots  | Lists missing text and what the game breaks |
| `TranslationParser::parse`                             | XML tables through roots    | Imports one language into JSON sources      |
| `TranslationInitializer::initialize`                   | JSON sources on a host path | Adds `null` for missing languages           |
| `TranslationFormatter::check_format`                   | JSON sources on host paths  | Reports noncanonical files; `format` fixes  |
| `TranslationSourceReader`, `TranslationGamedataReader` | Either layout through roots | Describe a project, with findings           |
| `TranslationEditor`                                    | One file of either kind     | Saves edits in place or as a transaction    |

- Each roots-based operation has an `_in` twin (`build_roots_in`, `verify_roots_in`, `parse_in`, the readers' `read_in`)
  taking an `XraySearch` you already opened, so archived tables read like loose ones.
- `build_file` and `verify_file` take one host file.
- Options carry a `JobHandle` for progress and cancellation. A cancelled run reports `JobOutcome::Cancelled`; check the
  outcome before reporting a run as complete.

## Build game string tables

```rust,no_run
use xrf_translation::{TranslationBuildOptions, TranslationBuilder, TranslationLanguage};
use xrf_vfs::{XrayMountMode, XrayRoots};

fn main() -> xrf_error::XrfResult {
  let roots = XrayRoots::one("sources".into(), XrayMountMode::Directory);
  let options = TranslationBuildOptions {
    job: Default::default(),
    output: Default::default(),
    is_sorted: true,
    output_dir: "output/text".into(),
    language: TranslationLanguage::English,
    link_policy: Default::default(),
  };

  let result = TranslationBuilder::build_roots(&roots, Some("translations"), &options)?;

  println!(
    "{} sources, {} tables; {:?}",
    result.sources, result.files, result.outcome
  );

  Ok(())
}
```

This reads sources below `sources/translations` and writes `output/text/eng/<source name>.xml` for each.

- Tables are written flat, whatever directory a source sits in. `CStringTable::Init` lists `text\<language>\*.xml` and
  opens each by bare name, so a nested table would stop the game with "Can't find specified xml file".
- `TranslationLanguage::All` builds every supported language.
- `is_sorted` orders entries naturally by ID, as a formatted source does; otherwise they keep the source's order.
- Missing translations build to the ID itself. A successful build does not prove every string is translated; run the
  verifier for that.

Before writing anything, a build refuses:

- output inside its source roots;
- two sources that would write the same table, including two with one name in different directories.

These entries fail the build, naming the entry:

- a value the language's code page cannot hold;
- a line longer than the game's XML reader accepts: 4094 bytes, the stricter of xray-16's 4095 and Anomaly's 4094. A
  longer one is fatal in game.

A cancelled build leaves the tables it wrote valid and the rest absent.

## Verify

`TranslationVerifier` reports two checks:

- **`translations`** (completeness): every ID a requested language has no text for, counted always and listed when
  `is_detailed`.
- **`translations.engine`**: what the game would refuse or show wrong, listed in full in `problems`:

| Rule                                  | Meaning                                                             |
| ------------------------------------- | ------------------------------------------------------------------- |
| `translations.unencodable`            | A character the language's code page lacks                          |
| `translations.overlong`               | A line past the XML reader's limit                                  |
| `translations.duplicate-across-files` | An ID two sources define, which xray-16 resolves by a parallel load |
| `translations.collision`              | Two sources that build to one table                                 |

`to_report` holds both checks; `to_engine_report` only the engine check. The CLI decides on the engine check (both
under `--strict`), because the game shows a missing translation as its ID but does not load an overlong line.

## Import

`TranslationParser::parse` imports one explicitly chosen language from raw XML.

- `TranslationParseOptions` carries the roots, optional prefix and file name, output directory, overwrite policy, and
  dry-run flag.
- `All` is not a valid import language.
- Existing different text is kept unless `is_overwrite` is on.
- `is_dry_run` shows the merge result without writing.
- A run that finds no tables is an error, not an empty success.
- Individual unreadable tables become findings; failing to mount the roots or write a target is still an error. Check
  `get_status` and `get_findings` even when the call returns `Ok`.

## Scaffold and format

- `TranslationInitializer` adds `null` entries for missing languages. It does not translate, and rewrites only a source
  that gained something.
- `TranslationFormatter` writes the canonical form every writer here shares: IDs and languages in natural order,
  two-space indentation, a trailing newline, and the file's own line endings unless the options name others. Check mode
  reports whether a rewrite is needed without changing files.

## Edit

Edits are `TranslationEdit::Set` or `Remove`, applied to one language.

- `TranslationEditor::apply_edits` rewrites a host file in place.
- `render_edits` returns the edited bytes for a file addressed by logical path.
- `build_save_transaction` gathers a file's edits for every language into one `XrayTransaction` over a search, refusing
  an archived copy.
- An XML edit that would leave a line the game refuses is refused; a long line already in the file is not.
- `TranslationProjectDescriptor::find_unwritable` says why a value cannot be written in a language: a character its
  code page lacks, or a line too long.

## Project layouts

| Reader                      | Mode       | Reads                               |
| --------------------------- | ---------- | ----------------------------------- |
| `TranslationSourceReader`   | `Source`   | JSON under `translations`           |
| `TranslationGamedataReader` | `Gamedata` | XML under `configs\text\<language>` |

- Both report unreadable files and duplicate IDs as findings and keep reading.
- The gamedata reader also reports an unknown language directory, and a table nested below a language directory
  (`translations.nested`), whose strings never load.
- A descriptor records each language's copy of each file by logical path and, when loose, by physical path. The
  physical path is for display; its absence means the copy is archived and cannot be edited.
- `TranslationProjectMode::detect` suggests a layout; the caller still chooses.

### Reading game tables

Game tables are read as tolerantly as the engine reads them. The encoding is chosen by:

1. a byte order mark;
2. otherwise, a language directory XRF knows, whatever the file declares. No engine reads the declaration, and shipped
   tables contradict theirs: Clear Sky's Polish, Czech, Hungarian and German tables all declare Windows-1251;
3. otherwise, the file's declaration;
4. otherwise, Windows-1251, the unprefixed fonts' code page.

UTF-16 is refused. Writes keep the encoding and byte order mark a file was read with.

## Errors

- Source parse, encoding and publication failures return `Err`.
- Per-file problems in a whole-tree read or import are findings beside the result.

## Module map

| Module     | Holds                                                                                                     |
| ---------- | --------------------------------------------------------------------------------------------------------- |
| `text`     | What a translation is, whichever file holds it                                                            |
| `language` | The languages and the code pages they are drawn in                                                        |
| `xml`      | The XML format end to end, including the line limit                                                       |
| `json`     | The JSON format end to end                                                                                |
| `project`  | Whole trees: `describe`, the `build`, `verify`, `parse`, `initialize` and `format` passes, and the editor |

## Checks

Run `cargo test --locked -p xrf-translation` from the repository root. The optional `typescript-bindings` feature
derives `specta` types for the frontend's generated bindings.

Sources: the [public API](src/lib.rs), [operations](src/project), and [language definitions](src/language).
