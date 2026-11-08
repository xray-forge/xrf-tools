# xrf-dialog

Reads S.T.A.L.K.E.R. dialog XML into conversations, phrases, links and script elements, with source ranges and parse
issues. It feeds the desktop app's dialog viewer and `xrf-cli dialog`.

Dialogs live in `dialog*.xml` under `configs\gameplay`. Each `dialog` holds phrases, and each phrase names:

- a translation key for its line;
- the phrases that may follow it;
- the script calls and info portions (story flags) that gate it or run after it.

The crate reads this data. It does not run dialog scripts or simulate game conditions.

## Read one file

```rust
use xrf_dialog::DialogFile;

fn main() -> xrf_error::XrfResult {
  let file = DialogFile::read_from_bytes(
    br#"<game_dialogs>
    <dialog id="greeting"><phrase_list>
      <phrase id="0"><text>greeting_text</text><next>1</next></phrase>
      <phrase id="1"><text>reply_text</text></phrase>
    </phrase_list></dialog>
  </game_dialogs>"#,
  )?;

  let dialog = file.find_dialog("greeting").expect("greeting dialog");

  assert_eq!(dialog.find_phrase("0").unwrap().list_next(), ["1"]);
  assert_eq!(dialog.find_phrase("1").unwrap().get_text(), Some("reply_text"));
  assert!(file.get_issues().is_empty());

  Ok(())
}
```

- Phrase IDs are labels within one dialog. Phrase `0` is the entry phrase.
- `next` links keep their authored order, which is the order the player sees the choices in.
- A phrase's text is a translation key; an empty `<text>` reads as no key. A single file does not resolve keys to lines
  (a project does).
- Every child element is kept in order as a `DialogElement`. It is classified by `DialogElementKind` but keeps its name,
  so an unknown element is reported rather than dropped.
- Phrases may sit inside a `phrase_list` or directly under their dialog; `is_in_phrase_list` records which.

## Encoding and source ranges

`DialogFile` keeps its decoded source, the encoding and any byte order mark, so a rewrite can reproduce the same bytes.

The encoding is chosen in this order:

1. A UTF-8 byte order mark.
2. The XML declaration.
3. Windows-1251, when there is neither.

UTF-16 is refused.

Dialog, phrase, element and issue ranges are byte offsets into the decoded source, not into the original file. Keep the
source with its ranges when displaying or splicing.

## Open a project

Choose a `DialogProjectMode`, build a `DialogProjectLayout`, then call `DialogProject::open` with the roots, or
`from_search` over a search you already mounted. Reads go through the VFS, so archived files read like loose ones.

```rust,no_run
use xrf_dialog::{DialogProject, DialogProjectLayout, DialogProjectMode};
use xrf_vfs::{XrayMountMode, XrayRoots};

fn main() -> xrf_error::XrfResult {
  let roots = XrayRoots::one("gamedata".into(), XrayMountMode::Directory);
  let layout = DialogProjectLayout::new(DialogProjectMode::Gamedata);
  let project = DialogProject::open(&roots, &layout)?;

  println!(
    "{} dialogs; {} findings",
    project.sum_dialogs(),
    project.get_findings().len()
  );

  Ok(())
}
```

Both modes read `dialog*.xml` from `configs\gameplay`. The mode only decides where the text comes from:

| Mode                 | Text source                                      |
| -------------------- | ------------------------------------------------ |
| `Gamedata` (default) | The per-language string tables in `configs\text` |
| `Source`             | XRF's translation JSON under `translations`      |

- The layout's optional prefixes override these locations. `is_text_skipped` skips the text for a caller that shows no
  lines.
- `DialogProjectMode::detect` and `detect_in` suggest a mode from the roots' contents. The layout you pass to `open`
  still decides.
- `open` fails when the roots hold no dialog files under the prefix.

### Query a project

- `get_files` and `get_findings` give the parsed files and their problems.
- `describe` summarizes the project. `describe_dialog` returns one conversation with its lines resolved through the
  project's `DialogTextIndex`.
- A dialog is addressed by file and id, because the same id may be declared in several files.
  `list_files_declaring` names them.
- `list_references` lists the dialog and phrase elements a predicate accepts, such as every script call.
- `DialogProjectFile::is_editable` and `DialogProject::is_editable` say whether a file, or the whole project, can be
  written back. An archived file cannot.

How text resolves:

- With no language named, the tree's first language is used; a project may not ship English.
- A language the tree does not hold resolves no line, rather than falling back to another.
- A key defined in two text files resolves to the last one read, as the engine's string table does.

A condition on the entry phrase is marked ignored unless some phrase leads back to it, because only then does the engine
read it.

## Find who offers a dialog

`DialogPartners` reads the character descriptions, profiles and info portions beside the dialogs. It answers which
dialogs reach the actor when it talks to an NPC of a given profile, as `CActor::UpdateAvailableDialogs` gathers them.

```rust,no_run
use xrf_dialog::{DialogPartners, DialogProject, DialogProjectLayout};
use xrf_vfs::{XrayMountMode, XrayRoots};

fn main() -> xrf_error::XrfResult {
  let roots = XrayRoots::one("gamedata".into(), XrayMountMode::Directory);
  let project = DialogProject::open(&roots, &DialogProjectLayout::default())?;
  let partners = DialogPartners::from_search(project.get_search(), project.get_dialogs_prefix())?;
  let profile = partners.find_profile("zat_b33_stalker_snag").expect("a profile");

  for (dialog, offers) in partners.list_offers(profile) {
    println!("{dialog}: {offers:?}");
  }

  Ok(())
}
```

- A profile resolves to its pinned character, or to every character its class may pick. The engine narrows that at
  spawn, so the offers are a superset.
- Offers list start dialogs first, then the characters' actor dialogs, then dialogs opened by info portions. A dialog
  reached two ways lists both.
- Files are chosen by name (`character_desc*`, `npc_profile*`, `info_*`) and read with `#include` lines expanded,
  relative to `configs` as the engine's `$game_config$`. A file another one includes is read only through it.

## Issues, findings and errors

Reading never fails on content the schema does not describe. Check `get_issues` as well as the dialogs. It reports:

- unknown elements and attributes;
- a `dialog` or `phrase` without an id, which is skipped since nothing can address it;
- a `priority` that is not an integer; the dialog is kept without one.

A dialog with no phrases is not malformed: a script may build them at runtime.

A project turns issues and unreadable files into `DialogFinding`s (`dialog.schema`, `dialog.unreadable`, and
`dialog.text.*` for its text) and keeps reading. `DialogPartners` reports an unreadable file and a repeated id (the
engine keeps the first) the same way.

File, encoding and XML failures of a single document return `Err`.

## Module map

| Module         | Holds                                                                         |
| -------------- | ----------------------------------------------------------------------------- |
| `conversation` | One conversation as written: dialog, phrases, elements, parse issues          |
| `dialog_file`  | One document and the source its ranges address, decoded by `dialog_source`    |
| `project`      | A whole tree, its wire `descriptor`s, and the `text` tree lines resolve in    |
| `partner`      | Characters, profiles and info portions deciding which dialogs reach the actor |

## Checks

Run `cargo test --locked -p xrf-dialog` from the repository root.

Sources: the [file model](src/dialog_file.rs), [project API](src/project/dialog_project.rs),
[partners](src/partner/dialog_partners.rs), and [tests](src/tests).
