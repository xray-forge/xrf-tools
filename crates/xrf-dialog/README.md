# xrf-dialog

Reads S.T.A.L.K.E.R. dialog XML as conversations, phrases, links, and script elements, with source ranges and parse
issues for inspection.

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
  assert!(file.get_issues().is_empty());

  Ok(())
}
```

Phrase IDs are labels within one dialog. `next` links retain their authored order, which is the order of the player's
choices. A phrase's text names a translation entry; parsing dialog XML alone does not resolve that entry.

## Open a project

Use `DialogFile::read_from_bytes` for one document. For a project, choose a `DialogProjectMode`, construct a
`DialogProjectLayout`, then call `DialogProject::open` with the roots and layout. `detect_mode` suggests a mode from
the roots' contents. Project reads use the VFS, including archived dialog files.

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

`get_files` and `get_findings` expose parsed content and problems. `describe` summarizes the project;
`describe_dialog` returns one conversation with optional translated text from its `DialogTextIndex`.

Choose source mode for the XRF source layout and gamedata mode for game string tables. The layout's optional dialog
and translation prefixes override its defaults. Mode detection is advisory; the layout passed to `open` selects the
interpretation.

## Find who offers a dialog

`DialogPartners` reads the character descriptions, profiles and info portions beside the dialogs, and answers what
reaches the actor when it talks to an NPC of a profile, the way `CActor::UpdateAvailableDialogs` gathers it.

```rust,no_run
use xrf_dialog::{DialogPartners, DialogProject, DialogProjectLayout};
use xrf_vfs::{XrayMountMode, XrayRoots};

fn main() -> xrf_error::XrfResult {
  let roots = XrayRoots::one("gamedata".into(), XrayMountMode::Directory);
  let project = DialogProject::open(&roots, &DialogProjectLayout::default())?;
  let partners = DialogPartners::from_vfs(project.get_vfs(), project.get_dialogs_prefix())?;
  let profile = partners.find_profile("zat_b33_stalker_snag").expect("a profile");

  for (dialog, offers) in partners.list_offers(profile) {
    println!("{dialog}: {offers:?}");
  }

  Ok(())
}
```

A profile resolves to its pinned specific character, or to every character it may pick by class; the engine narrows
the candidates further at spawn, so the offers are the superset. Files are chosen by name (`character_desc*`,
`npc_profile*`, `info_*`) and read with their `#include` lines expanded, and a file another one includes is read only
through it. A file that cannot be read, and a second declaration of an id, become findings.

## Source ranges and issues

Project discovery selects dialog XML by filename convention, so it does not treat every gameplay XML file as a dialog.
Source ranges refer to the retained decoded text. An archived file has no writable physical path.

`DialogFile` retains its decoded source, encoding, and BOM metadata. Dialog and phrase ranges address bytes in that
decoded source, not offsets in the original legacy-encoded file. Keep the source with its ranges when displaying it.

A dialog without phrases may build them from script at runtime and is not inherently malformed. The parser preserves
inspectable content and records supported structural issues; callers should read `get_issues` as well as the dialogs.

## Errors and checks

File or XML read failures are returned through `XrfResult`; project findings and per-file parse issues carry problems
that can be reported beside readable content. The crate does not execute dialog scripts or simulate runtime conditions.

Run `cargo test --locked -p xrf-dialog` from the repository root.

See the [file model](src/file.rs), [project API](src/project/dialog_project.rs), and [tests](src/tests).
