use std::path::Path;

use crate::project::describe::translation_finding::TranslationFinding;

/// One thing a translation run did or found, handed to its listener as it happens.
///
/// One vocabulary across the build, format, initialize, parse and verify runs, so a surface watching any of them reads
/// the same words. Borrowed from the run, so emitting one costs a call; a listener that keeps one copies what it needs.
#[derive(Clone, Copy, Debug)]
pub enum TranslationEvent<'e> {
  /// A path passed over because it is not a source the run reads.
  FileSkipped { path: &'e Path },
  /// A build found its sources and is about to compile them.
  BuildStarted { sources: usize },
  /// A build is compiling one source, named as messages name it.
  SourceBuilding { subject: &'e str },
  /// A build is writing one string table.
  TableWriting { path: &'e Path },
  /// A format run or check selected its sources.
  FormatStarted { files: usize, is_check: bool },
  /// A format run or check judged one source: already canonical, reported by a check, or rewritten.
  SourceFormatted {
    path: &'e Path,
    is_changed: bool,
    is_check: bool,
  },
  /// An initialization is walking a directory.
  InitializeStarted { dir: &'e Path },
  /// An initialization gave an id an explicit empty entry for a language.
  KeyInitialized { id: &'e str, language: &'e str },
  /// An import resolved where it reads from, described as the run names it.
  ParseStarted {
    scope: &'e str,
    language: &'e str,
    is_dry_run: bool,
  },
  /// An import left a source as it was, because merging changed nothing.
  SourceUnchanged { path: &'e Path },
  /// An import created or updated a source, or would have in a dry run.
  SourceWritten {
    path: &'e Path,
    is_new: bool,
    is_dry_run: bool,
  },
  /// A verification found its sources and is about to judge them.
  VerifyStarted { sources: usize },
  /// A verification found an id with no text in a language.
  KeyMissing {
    key: &'e str,
    language: &'e str,
    subject: &'e str,
  },
  /// A verification found something in one source the game would refuse or mangle.
  ProblemFound {
    subject: &'e str,
    problem: &'e TranslationFinding,
  },
}
