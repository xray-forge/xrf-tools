# xrf-translation

Reads, edits, verifies, and builds S.T.A.L.K.E.R. translations as multilingual JSON sources and per-language XML string
tables.

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

Arrays become the engine's literal `\n` line-break sequence on build. A missing or null translation is different from
an empty string. Language keys use codes such as `eng`, `rus`, and `ukr`; the game output is a per-language XML table.

## Choose an operation

- `TranslationParser::parse` imports one language's XML tables into JSON sources; its result includes skipped-file
  findings, so inspect the status even when the call returns `Ok`.
- `TranslationBuilder::build_roots` compiles JSON sources into game string tables. Output must be outside the input
  roots; colliding targets are rejected before writing.
- `TranslationVerifier::verify_roots` checks translation sources.
- `TranslationInitializer::initialize` adds explicit `null` entries for missing language variants.
- `TranslationFormatter::check_format` reports noncanonical JSON; `format` rewrites it.

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
  };

  let result = TranslationBuilder::build_roots(&roots, Some("translations"), &options)?;

  println!(
    "{} sources, {} tables; {:?}",
    result.sources, result.files, result.outcome
  );

  Ok(())
}
```

This reads sources below `sources/translations` and writes the selected language under the output directory. Use
`TranslationLanguage::All` to build every supported language. Missing translations build to the ID itself; run the
verifier when completeness matters, rather than treating a successful build as proof that every string is translated.

## Import and edit

`TranslationParser::parse` imports one explicitly selected language from raw XML. `TranslationParseOptions` carries
the roots, optional prefix and filename, output directory, overwrite policy, and dry-run flag. `All` is not a valid
import language. Existing different text is preserved unless `is_overwrite` is enabled.

Use `is_dry_run` to inspect the merge result before writing. Individual unreadable tables become findings; failure to
mount the roots or write a target still returns an error. Inspect `get_status` and `get_findings` on the result.

`TranslationInitializer` scaffolds null entries; it does not generate translated text. `TranslationFormatter` uses the
same canonical JSON writer as other edits. Its check mode reports whether a rewrite is needed without changing files.

## Verify completeness

`TranslationVerifier::verify_roots` checks the requested language or languages. Set `is_detailed` only when per-ID
findings are needed; summary counts are still produced without them. The `_in` variants of project operations accept
a VFS already mounted by the caller.

## Encodings, errors, and checks

Project reads use the VFS. Writes target host files; `apply_edits_to_asset` requires a writable asset. Language-specific
encoding rules apply when writing game tables, so not every Unicode character can be saved in every language.

Builds reject colliding targets and output inside their source roots. Cancellation can leave completed tables in place;
read the operation outcome before reporting the run as complete. Source parse, encoding, and publication failures are
returned through `XrfResult`.

Run `cargo test --locked -p xrf-translation` from the repository root. The optional `typescript-bindings` feature adds
frontend type metadata.

See the [public API](src/lib.rs), [operation options](src/project), and [language definitions](src/language.rs).
