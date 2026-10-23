# xrf-translation

Reads, edits, verifies, and builds S.T.A.L.K.E.R. translations as multilingual JSON sources and per-language XML string
tables.

The game shows text through string tables: XML files under `configs\text\<language>`, each mapping a string ID to one
language's text. XRF authors the same text as JSON sources, one file per table carrying every language, and builds
the string tables from them. This crate owns both formats, the import from game tables to sources, and the read model
the desktop app's translation editor and [xrf-dialog](../xrf-dialog/README.md) resolve text through.

## Source and game formats

A JSON source maps each string ID to language variants. A variant can be a string, an array of lines, or `null`:

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

Arrays become the engine's literal `\n` line-break sequence on build:

```rust
use xrf_translation::TranslationVariant;

let lines = TranslationVariant::MultiString(vec![String::from("Find the guide."), String::from("Return before dark.")]);

assert_eq!(lines.to_single_line(), "Find the guide.\\nReturn before dark.");
```

A missing or `null` translation is different from an empty string: the engine shows the ID itself for it. Language
keys are the codes of `TranslationLanguage`: `eng`, `fra`, `ger`, `ita`, `pol`, `rus`, `spa`, and `ukr`. A source file
is any `*.json` (matched without case), and reading one refuses a repeated ID or language.

Each language's string table is written in that language's code page: Windows-1251 for `rus` and `ukr`, Windows-1250
for `ger` and `pol`, Windows-1252 for the rest. Not every Unicode character can be saved in every language.

## Choose an operation

| Operation                            | Reads                       | Does                                       |
| ------------------------------------ | --------------------------- | ------------------------------------------ |
| `TranslationBuilder::build_roots`    | JSON sources through roots  | Compiles game string tables                |
| `TranslationVerifier::verify_roots`  | JSON sources through roots  | Lists IDs a language has no text for       |
| `TranslationParser::parse`           | XML tables through roots    | Imports one language into JSON sources     |
| `TranslationInitializer::initialize` | JSON sources on a host path | Adds `null` for missing languages          |
| `TranslationFormatter::check_format` | JSON sources on host paths  | Reports noncanonical files; `format` fixes |
| `read_source`, `read_gamedata`       | Either layout through roots | Describes a project, with findings         |

Each roots-based operation has an `_in` twin (`build_roots_in`, `verify_roots_in`, `parse_in`, `read_source_in`,
`read_gamedata_in`) that takes an `XraySearch` the caller already opened, so archived tables read like loose ones.
`build_file` and `verify_file` take one host file. Options carry a `JobHandle` for progress and cancellation; a
cancelled run reports `JobOutcome::Cancelled` in its result, so read the outcome before reporting a run as complete.

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

This reads sources below `sources/translations` and writes `output/text/eng/<source name>.xml` for each, keeping the
source's subdirectory. Use `TranslationLanguage::All` to build every supported language. `is_sorted` orders entries by
ID; otherwise they keep the source's order. Missing translations build to the ID itself, so run the verifier when
completeness matters rather than treating a successful build as proof that every string is translated.

A build refuses output inside its source roots and two sources that would write the same table, before writing
anything. A value its language's code page cannot hold fails the build with the entry named. A cancelled build leaves
the tables it had written valid and the rest absent.

## Import and edit

`TranslationParser::parse` imports one explicitly selected language from raw XML. `TranslationParseOptions` carries
the roots, optional prefix and file name, output directory, overwrite policy, and dry-run flag. `All` is not a valid
import language. Existing different text is preserved unless `is_overwrite` is enabled. A run that finds no tables is
an error rather than an empty success.

Use `is_dry_run` to inspect the merge result before writing. Individual unreadable tables become findings; failure to
mount the roots or write a target still returns an error. Inspect `get_status` and `get_findings` on the result even
when the call returns `Ok`.

`TranslationInitializer` scaffolds `null` entries; it does not generate translated text, and it rewrites only a source
that gained something. `TranslationFormatter` writes the canonical form every writer here shares: IDs and languages in
natural order, two-space indentation, a trailing newline, and the file's own line endings unless the options name
others. Its check mode reports whether a rewrite is needed without changing files.

Edits are `TranslationEdit::Set` or `Remove`, applied to one language. `apply_edits` rewrites a host file in place.
`render_edits` returns the edited bytes for a file addressed by logical path, and `build_save_transaction` gathers a
file's edits for every language into one `XrayTransaction` over a search, refusing an archived copy.
`find_unwritable_character` checks a value against the code page a project declared for a language.

## Project layouts

`read_source` describes an XRF source tree (`TranslationProjectMode::Source`, JSON under `translations`) and
`read_gamedata` a shipped one (`Gamedata`, XML under `configs\text\<language>`). Both report unreadable files and
duplicate IDs as findings and keep reading; `read_gamedata` also reports a language directory it does not know. A
descriptor records each language's copy of each file by logical path and, when it is loose, by physical path; the
physical path is for display, and its absence means the copy is archived and cannot be edited. `detect_mode` suggests
a layout; the caller still chooses it.

Game string tables are read the way the engine tolerates them. A byte order mark decides the encoding, then the XML
declaration, then the language directory's code page, then Windows-1252; UTF-16 is refused. Writes keep the encoding
and byte order mark a file was read with.

## Errors and checks

Source parse, encoding, and publication failures are returned through `XrfResult`; per-file problems in a whole-tree
read or import are findings beside the result.

Run `cargo test --locked -p xrf-translation` from the repository root. The optional `typescript-bindings` feature
derives `specta` types for the frontend's generated bindings.

## Module map

The modules follow the data rather than the operations: `xml` and `json` each own one file format end to end, and
`project` owns whole trees of them: discovery, description, editing, and the build, verify, initialize and format
passes.

See the [public API](src/lib.rs), [operations](src/project), and [language definitions](src/language.rs).
