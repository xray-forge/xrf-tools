use std::path::PathBuf;

use clap::{ArgMatches, Command};
use xrf_job::{ExecutionOrigin, ExecutionRequest};
use xrf_output::OutputVerbosity;
use xrf_utils::HostLinkPolicy;
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::core::arguments::{JobsArgument, LinkPolicyArgument, ReportingArguments, XrayRootsArguments};
use crate::core::report::{ReportDestination, ReportingOptions};

fn parse_reporting(arguments: &[&str]) -> Result<ArgMatches, clap::Error> {
  ReportingArguments::declare(Command::new("xrf-cli").no_binary_name(true)).try_get_matches_from(arguments)
}

/// The operation's own matches, from a root that carries the globals - the shape `application.rs` dispatches on, and
/// the only place a pair written at two levels meets.
fn parse_nested(root: &[&str], operation: &[&str]) -> ArgMatches {
  let matches: ArgMatches = ReportingArguments::declare(Command::new("xrf-cli").no_binary_name(true))
    .subcommand(Command::new("info"))
    .try_get_matches_from(root.iter().chain(["info"].iter()).chain(operation.iter()))
    .expect("a pair written at two levels to satisfy clap");

  matches
    .subcommand_matches("info")
    .expect("the operation to have matched")
    .clone()
}

#[test]
fn reads_the_destination_a_caller_selected() {
  assert_eq!(
    ReportingArguments::to_destination(&parse_reporting(&[]).unwrap()),
    ReportDestination::None
  );
  assert_eq!(
    ReportingArguments::to_destination(&parse_reporting(&["--json"]).unwrap()),
    ReportDestination::Stdout
  );
  assert_eq!(
    ReportingArguments::to_destination(&parse_reporting(&["--report", "out.json"]).unwrap()),
    ReportDestination::File(PathBuf::from("out.json"))
  );
}

#[test]
fn refuses_incompatible_output_selections() {
  assert!(parse_reporting(&["--json", "--report", "out.json"]).is_err());
  assert!(parse_reporting(&["--silent", "--verbose"]).is_err());
}

/// What clap refuses on its own never reaches here; this is the same rule for the pair it cannot see, because
/// `conflicts_with` holds within one parse level only.
#[test]
fn names_a_conflicting_pair_however_it_was_written() {
  let find = ReportingArguments::find_conflicting_selection;

  assert_eq!(find(&parse_reporting(&[]).unwrap()), None);
  assert_eq!(find(&parse_reporting(&["--silent"]).unwrap()), None);
  assert_eq!(find(&parse_reporting(&["--json"]).unwrap()), None);
  assert_eq!(
    find(&parse_nested(&["--verbose"], &["--silent"])),
    Some(("--silent", "--verbose"))
  );
  assert_eq!(
    find(&parse_nested(&["--json"], &["--report", "out.json"])),
    Some(("--json", "--report <PATH>"))
  );
  assert_eq!(find(&parse_nested(&["--verbose"], &[])), None);
}

#[test]
fn maps_the_verbosity_flags() {
  assert_eq!(ReportingArguments::to_verbosity(true, false), OutputVerbosity::Silent);
  assert_eq!(ReportingArguments::to_verbosity(false, true), OutputVerbosity::Verbose);
  assert_eq!(ReportingArguments::to_verbosity(false, false), OutputVerbosity::Normal);
}

#[test]
fn resolves_reporting_options_together() {
  let options: ReportingOptions = ReportingArguments::of(&parse_reporting(&["--json", "--verbose"]).unwrap());

  assert_eq!(options.destination, ReportDestination::Stdout);
  assert_eq!(options.output.get_verbosity(), OutputVerbosity::Verbose);
}

fn parse_jobs(arguments: &[&str]) -> Result<ArgMatches, clap::Error> {
  JobsArgument::declare(Command::new("verify").no_binary_name(true)).try_get_matches_from(arguments)
}

fn parsed_jobs(arguments: &[&str]) -> Option<ExecutionRequest> {
  JobsArgument::of(&parse_jobs(arguments).expect("the arguments parse"))
}

#[test]
fn defaults_to_letting_the_machine_decide() {
  assert_eq!(parsed_jobs(&[]), Some(ExecutionRequest::Auto));
  assert_eq!(
    parsed_jobs(&[])
      .expect("a declared command always answers")
      .resolve()
      .get_origin(),
    ExecutionOrigin::Auto
  );
}

#[test]
fn reads_both_spellings_of_the_jobs_flag() {
  assert_eq!(parsed_jobs(&["--jobs", "4"]), parsed_jobs(&["-j", "4"]));
  assert_eq!(
    parsed_jobs(&["-j", "50%"]),
    Some("50%".parse().expect("a share parses"))
  );
}

/// A command that never declared the flag is not eligible, and says so rather than pretending to a default.
#[test]
fn answers_for_a_command_that_offers_no_say() {
  let matches: ArgMatches = Command::new("info")
    .no_binary_name(true)
    .try_get_matches_from::<_, &str>([])
    .expect("a command with no arguments parses");

  assert_eq!(JobsArgument::of(&matches), None);
}

/// Rejection is clap's, so an unreadable value ends the run the way every other bad argument does.
#[test]
fn refuses_an_unreadable_jobs_value_before_the_command_runs() {
  let rejected: clap::Error = parse_jobs(&["-j", "sequential"]).expect_err("the value is refused");

  assert_eq!(rejected.kind(), clap::error::ErrorKind::ValueValidation);
  assert!(rejected.to_string().contains("auto"));
}

#[test]
fn refuses_a_worker_count_nobody_meant() {
  for value in ["0", "0%", "150%", "100000"] {
    assert_eq!(
      parse_jobs(&["-j", value]).expect_err("the value is refused").kind(),
      clap::error::ErrorKind::ValueValidation,
      "{value} was accepted"
    );
  }
}

fn parse_link_policy(arguments: &[&str]) -> HostLinkPolicy {
  LinkPolicyArgument::of(
    &LinkPolicyArgument::declare(Command::new("unpack"))
      .try_get_matches_from(arguments)
      .expect("arguments parse"),
  )
}

#[test]
fn follows_links_unless_asked_to_refuse_them() {
  assert_eq!(parse_link_policy(&["unpack"]), HostLinkPolicy::Follow);
  assert_eq!(parse_link_policy(&["unpack", "--refuse-links"]), HostLinkPolicy::Refuse);
}

#[test]
fn follows_links_on_a_command_without_the_flag() {
  assert_eq!(
    LinkPolicyArgument::of(
      &Command::new("pack")
        .try_get_matches_from(["pack"])
        .expect("arguments parse")
    ),
    HostLinkPolicy::Follow
  );
}

fn parse_roots(arguments: &[&str]) -> Result<ArgMatches, clap::Error> {
  XrayRootsArguments::declare(Command::new("list").no_binary_name(true), "a root", "a subtree")
    .try_get_matches_from(arguments)
}

#[test]
fn layers_every_path_as_one_source_and_reads_the_prefix() {
  let matches: ArgMatches = parse_roots(&[
    "-p",
    "mod",
    "-p",
    "game",
    "--source",
    "directory",
    "--prefix",
    "configs",
  ])
  .expect("the arguments parse");
  let roots: XrayRoots = XrayRootsArguments::of(&matches).expect("the roots");

  assert_eq!(
    roots
      .roots
      .iter()
      .map(|root| (root.path.clone(), root.mode))
      .collect::<Vec<_>>(),
    vec![
      (PathBuf::from("mod"), XrayMountMode::Directory),
      (PathBuf::from("game"), XrayMountMode::Directory),
    ]
  );
  assert_eq!(XrayRootsArguments::prefix_of(&matches), Some("configs"));
}

#[test]
fn reads_the_containing_installation_unless_told_otherwise_and_refuses_other_modes() {
  let matches: ArgMatches = parse_roots(&["-p", "game"]).expect("the arguments parse");

  assert_eq!(
    XrayRootsArguments::source_of(&matches).expect("the source"),
    XrayMountMode::ContainingInstallation
  );
  assert_eq!(XrayRootsArguments::prefix_of(&matches), None);
  assert_eq!(
    parse_roots(&["-p", "game", "--source", "nowhere"])
      .expect_err("the mode is refused")
      .kind(),
    clap::error::ErrorKind::InvalidValue
  );
}
