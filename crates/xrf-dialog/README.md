# xrf-dialog

Reads S.T.A.L.K.E.R. dialog XML as conversations, phrases, links, and script elements, with source ranges and parse
issues for inspection.

Dialogs are the `dialog*.xml` files under `configs\gameplay`. A `dialog` holds phrases; each phrase names a translation
key for its line, the phrases that may follow it, and the script calls and info portions (story flags) that gate or
follow it. The crate reads that data for the desktop app's dialog viewer and `xrf-cli dialog`; it does not execute
dialog scripts or simulate runtime conditions.

## Read one conversation

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

Phrase IDs are labels within one dialog, and phrase `0` is the entry phrase the dialog opens with. `next` links keep
their authored order, which is the order the player sees the choices in. A phrase's text is a translation key, and an
empty `<text>` reads as no key; parsing dialog XML alone does not resolve the key to a line.

Every child element is kept in written order as a `DialogElement`, classified by `DialogElementKind` but keeping its
name, so an element the schema does not know survives and is reported rather than dropped. Phrases may sit inside a
`phrase_list` or directly under their dialog; `is_in_phrase_list` records which.

## Encoding and source ranges

`DialogFile` retains its decoded source, the encoding it was decoded with, and any byte order mark, so a rewrite can use
the same bytes. A UTF-8 byte order mark decides the encoding over the declaration; otherwise the XML declaration
decides, and a file without one is read as Windows-1251. UTF-16 is refused.

Dialog, phrase, element and issue ranges address bytes in the retained decoded source, not offsets in the original
legacy-encoded file. Keep the source with its ranges when displaying or splicing it.

## Open a project

For a project, choose a `DialogProjectMode`, construct a `DialogProjectLayout`, then call `DialogProject::open` with the
roots and layout, or `from_search` over a search the caller already mounted. Project reads go through the VFS, so
archived dialog files are read the same way as loose ones.

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

Both modes read dialogs from `configs\gameplay`, selecting files by name (`dialog*.xml`) rather than treating every
gameplay XML as a dialog. The mode decides only where dialog text sits: gamedata mode (the default) reads the
per-language string tables under `configs\text`, source mode the XRF JSON under `translations`. The layout's optional
prefixes override those defaults, and `is_text_skipped` leaves the text unread for a caller that shows no lines.
`DialogProjectMode::detect` and `detect_in` suggest a mode from the roots' contents; the suggestion is advisory, and the
layout passed to `open` selects the interpretation. `open` fails when the roots hold no dialog files under the prefix.

`get_files` and `get_findings` expose parsed content and problems. `describe` summarizes the project;
`describe_dialog` returns one conversation with its lines resolved through the project's `DialogTextIndex`. A dialog is
addressed by its file and its id, because one id may be declared in several files, each a dialog of its own;
`list_files_declaring` names them. With no language named, the text is the tree's first language, since a project may
ship one and not always English; a language the tree does not hold resolves no line rather than another's. A key two
text files define resolves to the last one read, as the engine's string table keeps it.

`list_references` lists the dialog and phrase elements a predicate accepts, such as every script call. A condition on
the entry phrase is marked ignored unless a phrase leads back to it, because the engine reads the entry phrase's
conditions only then.

An archived file has no physical path; `DialogProjectFile::is_editable` and `DialogProject::is_editable` say whether a
file, or the whole project, could be written back.

## Find who offers a dialog

`DialogPartners` reads the character descriptions, profiles and info portions beside the dialogs, and answers what
reaches the actor when it talks to an NPC of a profile, the way `CActor::UpdateAvailableDialogs` gathers it.

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

A profile resolves to its pinned specific character, or to every character it may pick by class; the engine narrows
the candidates further at spawn, so the offers are the superset. Offers list start dialogs first, then the characters'
actor dialogs, then dialogs opened by info portions, and a dialog reached two ways lists both. Files are chosen by name
(`character_desc*`, `npc_profile*`, `info_*`) and read with their `#include` lines expanded (included names are opened
under `configs`, as the engine's `$game_config$`), and a file another one includes is read only through it.

## Issues, findings, and errors

Reading never fails on content the schema does not describe. `get_issues` reports unknown elements and attributes, a
`dialog` or `phrase` without an id (skipped, since nothing can address it), and a `priority` that is not an integer
(the dialog is kept without one). A dialog without phrases may build them from script at runtime and is not malformed.
Read `get_issues` as well as the dialogs.

A project turns those issues and unreadable files into `DialogFinding`s (`dialog.schema`, `dialog.unreadable`, and
`dialog.text.*` for its text tree) and keeps reading; `DialogPartners` reports an unreadable file and a second
declaration of an id (the engine keeps the first) the same way. File, encoding, and XML failures of a single document
are returned through `XrfResult`.

## Module map

The modules follow the data. `conversation` is one conversation as written: the dialog, its phrases (one line each),
their child elements, and the parse issues reading them raised. `dialog_file` is one document together with the text
its ranges address, decoded by `dialog_source`. `project` is a whole tree of them, with the `descriptor`s it sends
across the wire and the `text` tree its lines resolve through. `partner` is who the actor talks to: the characters,
profiles and info portions that decide which dialogs reach it.

## Checks

Run `cargo test --locked -p xrf-dialog` from the repository root.

See the [file model](src/dialog_file.rs), [project API](src/project/dialog_project.rs),
[partners](src/partner/dialog_partners.rs), and [tests](src/tests).
