use std::collections::{HashMap, HashSet};

use xrf_job::JobHandle;

use crate::project::gamedata_project::GamedataProject;
use crate::project::scripts::gamedata_parsed_script::GamedataParsedScript;
use crate::project::scripts::scripts_verifier::ScriptsVerifier;

/// The functions a tree's scripts define for the engine to call by name, as `CScriptEngine::function_object` finds them.
///
/// The engine splits a name at its last dot into a namespace and a function, loads the script directly under `scripts`
/// the namespace's first part names when there is one, and looks the function up in the namespace's table; a name
/// without a dot is looked up in `_G`, which `_g.script` fills. xrf-engine fills those tables at runtime through
/// `extern`, so a name a script registers that way is defined too, wherever the script lies.
#[derive(Default)]
pub(crate) struct GamedataScriptFunctions {
  /// The functions each namespace's script defines, by the namespace lowercased, as the file system matches it.
  namespaces: HashMap<String, HashSet<String>>,
  /// Namespaces whose script did not read or parse, of which nothing is known.
  unread: HashSet<String>,
  /// Dotted names registered through `extern`, as written.
  registered: HashSet<String>,
  /// Whether a script did not read or parse, so the names it may register through `extern` are not known.
  is_registration_hidden: bool,
}

impl GamedataScriptFunctions {
  const SCRIPTS_DIRECTORY: &'static str = "scripts\\";
  const SCRIPT_EXTENSION: &'static str = ".script";
  /// The namespace a name without a dot is looked up in, and the script that fills it, lowercased.
  const GLOBAL_NAMESPACE: &'static str = "_g";

  /// Read and parse every runtime script the project holds, or `None` for a tree holding none, such as a mod relying
  /// on the game's scripts, of which nothing is known.
  pub(crate) fn read(project: &GamedataProject, job: &JobHandle) -> Option<Self> {
    let scripts: Vec<GamedataParsedScript> = ScriptsVerifier::parse_runtime_scripts(project, job);

    if scripts.is_empty() {
      return None;
    }

    let mut functions: Self = Self::default();

    for parsed in scripts {
      functions.add(parsed);
    }

    Some(functions)
  }

  /// Take in what one script defines: the names it registers, and the functions of the namespace it fills, if it lies
  /// directly under `scripts`.
  fn add(&mut self, parsed: GamedataParsedScript) {
    let namespace: Option<String> = Self::find_namespace(&parsed.path);

    let Ok(script) = parsed.script else {
      // The scripts check reports the script; neither what it defines nor what it registers is known.
      self.unread.extend(namespace);
      self.is_registration_hidden = true;

      return;
    };

    self.registered.extend(script.get_registered_names().iter().cloned());

    if let Some(namespace) = namespace {
      self
        .namespaces
        .entry(namespace)
        .or_default()
        .extend(script.get_global_functions().iter().cloned());
    }
  }

  /// Whether the engine would find nothing under a name, as far as the scripts tell. A name in a namespace whose script
  /// did not read is not judged, and neither is one no namespace defines while some unread script may register it.
  pub(crate) fn is_missing(&self, name: &str) -> bool {
    if self.registered.contains(name) {
      return false;
    }

    let (namespace, function) = name.rsplit_once('.').unwrap_or((Self::GLOBAL_NAMESPACE, name));
    let file: String = namespace.split('.').next().unwrap_or(namespace).to_ascii_lowercase();

    if self.unread.contains(&file) {
      return false;
    }

    let is_defined: bool = self
      .namespaces
      .get(&namespace.to_ascii_lowercase())
      .is_some_and(|functions| functions.contains(function));

    !is_defined && !self.is_registration_hidden
  }

  /// The namespace a script fills when the engine loads it: its name, for a script directly under `scripts`.
  fn find_namespace(logical_path: &str) -> Option<String> {
    let lowercase: String = logical_path.to_ascii_lowercase();
    let name: &str = lowercase
      .strip_prefix(Self::SCRIPTS_DIRECTORY)?
      .strip_suffix(Self::SCRIPT_EXTENSION)?;

    (!name.contains('\\')).then(|| name.to_owned())
  }
}
