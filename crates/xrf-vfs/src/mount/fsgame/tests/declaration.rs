use crate::{FsgameDeclaration, FsgameFile};

#[test]
fn parses_a_declaration_with_every_field() {
  let declaration: FsgameDeclaration =
    FsgameDeclaration::parse("$game_meshes$ = true | true | $game_data$ | meshes\\ | *.ogf;*.omf | Game Object files")
      .expect("line parses");

  assert_eq!(declaration.alias, "$game_meshes$");
  assert!(declaration.is_recursive);
  assert!(declaration.is_notified);
  assert_eq!(declaration.root_alias, "$game_data$");
  assert_eq!(declaration.get_addition_segment(), Some("meshes"));
  assert_eq!(declaration.extensions.as_deref(), Some("*.ogf;*.omf"));
  assert_eq!(declaration.description.as_deref(), Some("Game Object files"));
}

#[test]
fn parses_a_declaration_that_names_its_root_and_nothing_else() {
  // `$downloads$` is one of these: three fields, so the alias is the root path itself.
  let declaration: FsgameDeclaration =
    FsgameDeclaration::parse("$downloads$ = false | false | $app_data_root$").expect("line parses");

  assert!(!declaration.is_recursive);
  assert_eq!(declaration.get_addition_segment(), None);
  assert_eq!(declaration.root_alias, "$app_data_root$");
}

#[test]
fn treats_the_installation_root_as_an_alias_like_any_other() {
  let declaration: FsgameDeclaration =
    FsgameDeclaration::parse("$arch_dir$ = false | false | $fs_root$ | db\\").expect("line parses");

  assert_eq!(declaration.root_alias, FsgameFile::ROOT_ALIAS);
  assert_eq!(declaration.get_addition_segment(), Some("db"));
}

#[test]
fn tolerates_the_tabs_and_trailing_spaces_real_files_carry() {
  let declaration: FsgameDeclaration =
    FsgameDeclaration::parse("$textures_ui$ \t= true \t| true\t| $textures$\t | ui\\   ").expect("line parses");

  assert_eq!(declaration.alias, "$textures_ui$");
  assert_eq!(declaration.get_addition_segment(), Some("ui"));
}

#[test]
fn rejects_a_line_with_too_few_fields() {
  assert!(FsgameDeclaration::parse("$broken$ = true | false").is_err());
  assert!(FsgameDeclaration::parse("$broken$").is_err());
}

#[test]
fn skips_comments_and_blank_lines() {
  assert!(!FsgameDeclaration::is_declaration(";abbreviation = recurs | notif"));
  assert!(!FsgameDeclaration::is_declaration("   "));
  assert!(FsgameDeclaration::is_declaration(
    "$arch_dir$ = false | false | $fs_root$ | db\\"
  ));
}
