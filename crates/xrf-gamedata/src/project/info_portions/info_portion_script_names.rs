use std::collections::{HashMap, HashSet};

use crate::project::gamedata_project::GamedataProject;
use crate::project::info_portions::info_portion_place::InfoPortionPlace;
use crate::project::info_portions::info_portion_role::InfoPortionRole;
use crate::project::info_portions::info_portion_script_functions::InfoPortionScriptFunctions;
use crate::project::info_portions::info_portion_table::InfoPortionTable;
use crate::project::scripts::gamedata_parsed_script::GamedataParsedScript;

/// What a tree's scripts do with the names they spell out, by name.
///
/// A call `InfoPortionScriptFunctions` knows gives, takes or reads the names passed to it. Any other call, a table key
/// or value, and a name built onto a literal prefix, which may be any name the prefix opens, do something unknown with
/// them.
#[derive(Default)]
pub(crate) struct InfoPortionScriptNames<'a> {
  /// Each name a giver call passes, by the first script and call giving it.
  pub(crate) given: HashMap<&'a str, InfoPortionPlace>,
  /// The names a reader call passes.
  pub(crate) read: HashSet<&'a str>,
  /// The names a call of an unknown function passes, or a table keeps.
  pub(crate) unknown: HashSet<&'a str>,
  /// The prefixes scripts build names onto that open a family of names.
  pub(crate) prefixes: HashSet<&'a str>,
  /// Every name a call passes or a table keeps, whatever is done with it.
  pub(crate) named: HashSet<&'a str>,
  /// Scripts that did not read or parse, whose names are not known.
  pub(crate) unread_count: usize,
}

impl<'a> InfoPortionScriptNames<'a> {
  /// How many `_`-separated parts a concatenated prefix needs before it names a family of info portions.
  const NAME_PREFIX_PARTS: usize = 2;
  /// What ends a prefix opening a family of names, before the part a script appends.
  const NAME_SEPARATOR: char = '_';

  /// What the parsed scripts do with their names, a script that did not read or parse counted.
  pub(crate) fn read(project: &GamedataProject, scripts: &'a [GamedataParsedScript]) -> Self {
    let mut names: Self = Self::default();

    for script in scripts {
      let Ok(lua) = &script.script else {
        names.unread_count += 1;
        continue;
      };

      for call in lua.get_string_calls() {
        let role: Option<InfoPortionRole> = call.get_function().and_then(InfoPortionScriptFunctions::find_role);

        for name in call.get_arguments() {
          names.named.insert(name);

          match role {
            Some(InfoPortionRole::Given) => {
              names.given.entry(name).or_insert_with(|| InfoPortionPlace {
                path: InfoPortionTable::resolve_report_path(project, &script.path),
                context: format!("script call '{}'", call.get_function().unwrap_or_default()),
              });
            }
            Some(InfoPortionRole::Read) => {
              names.read.insert(name);
            }
            // A take neither gives nor reads.
            Some(_) => {}
            None => {
              names.unknown.insert(name);
            }
          }
        }
      }

      for value in lua.get_table_strings() {
        names.named.insert(value);
        names.unknown.insert(value);
      }

      names.prefixes.extend(
        lua
          .get_concatenation_prefixes()
          .iter()
          .map(String::as_str)
          .filter(|prefix| Self::is_name_prefix(prefix)),
      );
    }

    names
  }

  /// Whether scripts name a portion where what they do with it is unknown: in a call this table does not know, in a
  /// table, or as one of a family a prefix opens.
  pub(crate) fn is_unknown(&self, name: &str) -> bool {
    self.unknown.contains(name)
      || self
        .prefixes
        .iter()
        .any(|prefix| name.len() > prefix.len() && name.starts_with(prefix))
  }

  /// The names scripts spell out that are written as info portion names are, the ones a misspelling may have been
  /// meant as.
  pub(crate) fn list_identifiers(&self) -> impl Iterator<Item = &'a str> + '_ {
    self
      .named
      .iter()
      .copied()
      .filter(|name| Self::is_identifier(name) && name.contains(Self::NAME_SEPARATOR))
  }

  /// Whether a prefix a script concatenates onto opens a family of names rather than any text: an identifier of at
  /// least two parts ending at a separator, such as `zat_b51_ordered_item_` or `jup_b221_`. A separator, a word, a
  /// level's prefix or a part cut short, such as `_`, `layer_`, `jup_` or `zat_b`, would excuse every name it starts.
  fn is_name_prefix(prefix: &str) -> bool {
    Self::is_identifier(prefix)
      && prefix.ends_with(Self::NAME_SEPARATOR)
      && prefix
        .split(Self::NAME_SEPARATOR)
        .filter(|part| !part.is_empty())
        .count()
        >= Self::NAME_PREFIX_PARTS
  }

  /// Whether a string is written as info portion names are: ASCII letters, digits and underscores.
  fn is_identifier(name: &str) -> bool {
    !name.is_empty()
      && name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == Self::NAME_SEPARATOR)
  }
}
