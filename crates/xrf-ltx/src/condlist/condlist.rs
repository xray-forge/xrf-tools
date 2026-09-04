use xrf_error::XrfResult;

use super::condlist_branch::CondlistBranch;
use super::source_span::SourceSpan;

/// A parsed X-Ray condition list.
#[derive(Clone, Debug, PartialEq)]
pub struct Condlist {
  pub branches: Vec<CondlistBranch>,
}

impl Condlist {
  pub fn parse(value: &str) -> XrfResult<Condlist> {
    let mut branches: Vec<CondlistBranch> = Vec::new();
    let mut branch_start: usize = 0;

    for raw_branch in value.split(',') {
      let leading_whitespace: usize = raw_branch.len() - raw_branch.trim_start().len();
      let branch_offset: usize = branch_start + leading_whitespace;
      let branch: &str = raw_branch.trim();

      if branch.is_empty() {
        return Err(SourceSpan::parsing_error(branch_offset, "Expected a condlist branch"));
      }

      branches.push(CondlistBranch::parse(branch, branch_offset)?);

      branch_start += raw_branch.len() + 1;
    }

    Ok(Condlist { branches })
  }
}

#[cfg(test)]
mod tests {
  use super::super::condlist_branch::CondlistCondition;
  use super::Condlist;

  #[test]
  fn parses_xray_condlist_syntax() {
    let condlist = Condlist::parse("{+info -other =actor_on_level(pripyat)} enabled, %=set_active_task(test)%")
      .expect("Expected valid condlist");

    assert_eq!(condlist.branches.len(), 2);
    assert_eq!(condlist.branches[0].result.as_deref(), Some("enabled"));
    assert_eq!(condlist.branches[1].result, None);
    assert_eq!(condlist.branches[0].conditions.len(), 3);
    assert_eq!(condlist.branches[1].effects.len(), 1);
    assert!(matches!(
      condlist.branches[0].conditions[2],
      CondlistCondition::Function {
        ref name,
        expected: true,
        parameters: Some(ref parameters),
        ..
      } if name == "actor_on_level" && parameters == &["pripyat"]
    ));
  }

  #[test]
  fn accepts_xray_condition_only_and_spaced_function_calls() {
    let condlist = Condlist::parse("{+info}, fallback, {= spawn_corpse (snork : : target : )} section")
      .expect("Expected valid X-Ray condlist");

    assert_eq!(condlist.branches[0].result, None);
    assert!(condlist.branches[0].effects.is_empty());

    let CondlistCondition::Function {
      name,
      parameters: Some(parameters),
      ..
    } = &condlist.branches[2].conditions[0]
    else {
      panic!("Expected a spawn_corpse function condition");
    };

    assert_eq!(name, "spawn_corpse");
    assert_eq!(parameters, &["snork", "", "target", ""]);
  }

  #[test]
  fn accepts_effects_before_condlist_results() {
    let condlist = Condlist::parse("%+info% next_section, {+condition} %=play_sound(sound)% another_section")
      .expect("Expected valid X-Ray effect-first condlist branches");

    assert_eq!(condlist.branches[0].result.as_deref(), Some("next_section"));
    assert_eq!(condlist.branches[0].effects.len(), 1);
    assert_eq!(condlist.branches[1].result.as_deref(), Some("another_section"));
    assert_eq!(condlist.branches[1].conditions.len(), 1);
    assert_eq!(condlist.branches[1].effects.len(), 1);
  }

  #[test]
  fn accepts_context_specific_condlist_results() {
    let condlist = Condlist::parse("15| guard, {=surge_started} | %+scene_end%")
      .expect("Expected valid context-specific condlist results");

    assert_eq!(condlist.branches[0].result.as_deref(), Some("15| guard"));
    assert_eq!(condlist.branches[1].result.as_deref(), Some("|"));
    assert_eq!(condlist.branches[1].effects.len(), 1);
  }

  #[test]
  fn rejects_malformed_condlist_syntax() {
    for value in [
      "",
      "   ",
      ", target",
      "target,",
      "first,,second",
      "{+info target",
      "{+} target",
      "{}",
      "target %effect",
      "{=check(foo} target",
      "{=check()tail} target",
      "{~not-a-number} target",
      "{+first} one {+second} two",
      "%+first% one %+second%",
    ] {
      assert!(Condlist::parse(value).is_err(), "Expected invalid condlist: {value}");
    }
  }
}
