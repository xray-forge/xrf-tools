use xrf_ltx::{Condlist, CondlistCondition, Ltx};

use crate::project::info_portions::info_portion_role::InfoPortionRole;

/// One mention of an info portion inside a file, before the file is known: the portion, what is done with it, and what
/// in the file names it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InfoPortionMention {
  pub(crate) name: String,
  pub(crate) role: InfoPortionRole,
  pub(crate) context: String,
}

impl InfoPortionMention {
  /// What separates a logic field's arguments from its condlists, as `xr_logic` reads `on_signal = signal | condlist`,
  /// `on_timer = time | condlist`, and `hit_on_bone = bone | condlist | bone | condlist`.
  const SEGMENT_SEPARATOR: char = '|';

  /// Every info portion the condlists of one config read, give, take or pass to a function, each named by its section
  /// and key, under `owner` when the config is an object's custom data.
  ///
  /// A value is split at every `|`, and each segment holding `{` or `%`, the brackets that carry info portions, is read
  /// as a condlist of its own; a segment that does not parse names none.
  pub(crate) fn list_in_ltx(ltx: &Ltx, owner: Option<&str>) -> Vec<Self> {
    let mut mentions: Vec<Self> = Vec::new();

    for (section_name, section) in ltx {
      for (key, value) in section {
        if !value.contains(['{', '%']) {
          continue;
        }

        let context: String = match owner {
          Some(owner) => format!("{owner} [{section_name}] {key}"),
          None => format!("[{section_name}] {key}"),
        };

        for segment in value
          .split(Self::SEGMENT_SEPARATOR)
          .filter(|segment| segment.contains(['{', '%']))
        {
          if let Ok(condlist) = Condlist::parse(segment) {
            Self::add_condlist(&condlist, &context, &mut mentions);
          }
        }
      }
    }

    mentions
  }

  pub(crate) fn new(name: &str, role: InfoPortionRole, context: &str) -> Self {
    Self {
      name: name.to_owned(),
      role,
      context: context.to_owned(),
    }
  }

  /// The mentions one condlist makes: its conditions read, its effects give or take, and the parameters of its
  /// functions, conditions and effects alike, are passed.
  fn add_condlist(condlist: &Condlist, context: &str, mentions: &mut Vec<Self>) {
    for branch in &condlist.branches {
      for condition in &branch.conditions {
        Self::add_condition(condition, InfoPortionRole::Read, context, mentions);
      }

      for effect in &branch.effects {
        let role: InfoPortionRole = match effect {
          CondlistCondition::InfoPortion { required: false, .. } => InfoPortionRole::Taken,
          _ => InfoPortionRole::Given,
        };

        Self::add_condition(effect, role, context, mentions);
      }
    }
  }

  /// The mention one condition or effect makes: an info portion in the role its place gives it, or each parameter of
  /// a function as passed.
  fn add_condition(condition: &CondlistCondition, role: InfoPortionRole, context: &str, mentions: &mut Vec<Self>) {
    match condition {
      CondlistCondition::InfoPortion { name, .. } => mentions.push(Self::new(name, role, context)),
      CondlistCondition::Function {
        parameters: Some(parameters),
        ..
      } => mentions.extend(
        parameters
          .iter()
          .map(|parameter| parameter.trim())
          .filter(|parameter| !parameter.is_empty())
          .map(|parameter| Self::new(parameter, InfoPortionRole::Passed, context)),
      ),
      _ => {}
    }
  }
}
