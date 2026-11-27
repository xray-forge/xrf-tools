use xrf_ltx::Condlist;
use xrf_random::{RandomAddress, RandomStream};

use crate::facts::simulation_facts::SimulationFacts;

fn pick(facts: &SimulationFacts, condlist: &str) -> Option<String> {
  let condlist: Condlist = Condlist::parse(condlist).expect("parses");

  facts
    .pick(&condlist, &mut RandomStream::new(1, RandomAddress::default()))
    .map(String::from)
}

#[test]
fn picks_the_first_branch_whose_info_portions_hold() {
  let mut facts: SimulationFacts = SimulationFacts::default();
  let spawn_num: &str = "{+jup_gathering} 0, {+zat_heli_3_searched -zat_heli_2_searched} 2, 1";

  assert_eq!(pick(&facts, spawn_num).as_deref(), Some("1"));

  facts.infos.insert(String::from("zat_heli_3_searched"));
  assert_eq!(pick(&facts, spawn_num).as_deref(), Some("2"));

  facts.infos.insert(String::from("jup_gathering"));
  assert_eq!(pick(&facts, spawn_num).as_deref(), Some("0"));
}

#[test]
fn answers_false_for_a_function_and_none_where_nothing_holds() {
  let facts: SimulationFacts = SimulationFacts::default();

  assert_eq!(pick(&facts, "{=surge_started} surge, calm").as_deref(), Some("calm"));
  assert_eq!(pick(&facts, "{!surge_started} calm, surge").as_deref(), Some("calm"));
  assert_eq!(pick(&facts, "{+absent} one").as_deref(), None);
  assert_eq!(pick(&facts, "{-absent} %=effect%").as_deref(), None);
}

#[test]
fn draws_one_number_for_every_probability_of_a_pick() {
  let facts: SimulationFacts = SimulationFacts::default();
  let condlist: Condlist = Condlist::parse("{~30} low, {~60} middle, high").expect("parses");
  let mut counts: [u32; 3] = [0; 3];

  for seed in 0..3_000 {
    match facts.pick(&condlist, &mut RandomStream::new(seed, RandomAddress::default())) {
      Some("low") => counts[0] += 1,
      Some("middle") => counts[1] += 1,
      _ => counts[2] += 1,
    }
  }

  // One shared draw: 30 % low, the next 30 % middle, 40 % high; two draws would make middle 42 %.
  assert!(counts[0].abs_diff(900) < 120, "{counts:?}");
  assert!(counts[1].abs_diff(900) < 120, "{counts:?}");
  assert!(counts[2].abs_diff(1_200) < 130, "{counts:?}");
}
