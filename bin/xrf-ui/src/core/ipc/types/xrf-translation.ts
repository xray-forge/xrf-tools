// Auto-generated rust bindings. Do not edit it manually.

import { JobOutcome } from "@/core/ipc/types/xrf-job";
import { Report } from "@/core/ipc/types/xrf-report";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** What one language's build produced. */
export type TranslationBuildLanguageSummary = {
  language: string;
  /** String tables written for this language, one per source. */
  files: number;
  /**
   * Ids compiled into them, counting an id once per file it appears in.
   *
   * A missing translation still compiles - to the id itself, which is the engine's own fallback - so
   * this counts what the tables hold rather than what was translated.
   */
  entries: number;
};

/** What a build produced, across every language it built. */
export type TranslationBuildResult = {
  /**
   * Whether the run compiled every source or was stopped between them.
   *
   * A stopped build leaves the string tables it had already written valid and the rest simply absent, which is a
   * state running it again resolves.
   */
  outcome: JobOutcome;
  duration: number;
  /** Sources read, whatever they built into. */
  sources: number;
  /** String tables written, across every language. */
  files: number;
  /** Per language, in build order. */
  languages: Array<TranslationBuildLanguageSummary>;
};

/** Every `kind` the `TranslationEdit` union is told apart by, so a switch or a comparison names one. */
export enum ETranslationEdit {
  /** Replace the winning entry's value, or append the entry when the file has none. */
  SET = "set",
  /** Remove the entry entirely, shadowed duplicates included. */
  REMOVE = "remove",
}

/**
 * One change to a translation entry, in whichever kind of file holds it.
 *
 * Format-neutral on purpose. It used to live beside the XML writer and carry a bare `String`, which
 * quietly could not express what a JSON source already holds: an entry whose text is an array of
 * lines. Editing one of those flattened it on save, and roughly 190 entries across ten files in the
 * engine's own translations are that shape.
 */
export type TranslationEdit =
  /** Replace the winning entry's value, or append the entry when the file has none. */
  | { kind: "set"; id: string; value: TranslationVariant }
  /** Remove the entry entirely, shadowed duplicates included. */
  | { kind: "remove"; id: string };

/** One logical translation file, and where each language's copy of it lives. */
export type TranslationFile = {
  /**
   * Language to the source holding it. A JSON source lists every language it carries against the
   * same one.
   *
   * Editability is per language rather than per file: an installation layered under a loose tree can
   * serve one language from a `.db` volume and the next from disk.
   */
  sources: { [key in string]: TranslationSource };
  entries: { [key in string]: { [key in string]: TranslationVariant | null } };
};

/**
 * Something worth reporting about a file that was opened anyway.
 *
 * The reader refuses nothing on content: an editor that will not open the file you need to fix is
 * no use, and the build and verifier keep their own guards.
 */
export type TranslationFinding = {
  rule: string;
  subject: string | null;
  message: string;
};

/**
 * What one formatting run judged and changed.
 *
 * Field for field what `LtxProjectFormatResult` reports, so the two `format --check` commands answer a consumer in one
 * shape. Its own type rather than that one: a crate deposits its own result, and reaching into `xrf-ltx` for a shape
 * would tie a translation run's wire contract to an unrelated format's.
 *
 * No findings array, unlike this crate's parse and verify results. A formatter's answer is the list of files to fix,
 * and a finding would spend a subject, a rule id and a message carrying one bit each over a single rule.
 */
export type TranslationFormatResult = {
  /**
   * Whether the run reached the end of the set or was stopped between files.
   *
   * A stopped rewrite leaves the files it had already formatted formatted and the rest untouched, which is a state
   * running it again resolves. Nothing is removed and nothing is half-written.
   */
  outcome: JobOutcome;
  /** Everything the run took, measured from when its caller created the job handle. */
  duration: number;
  /**
   * How much of `duration` had already passed when the per-file work began.
   *
   * Walking the paths and selecting sources out of them happens before a single file is read, and on a cold filesystem
   * or a large tree that is most of the wait. Named rather than folded away, so the split stays readable.
   */
  startupDuration: number;
  /** Sources that were not canonical: rewritten by a normal run, reported by a check. */
  invalidFiles: number;
  /** Which ones, in selection order. */
  toFormat: Array<string>;
  totalFiles: number;
  validFiles: number;
};

/** A language a translation is written in, spelled on the wire as its code; `all` stands for every one at once. */
export enum ETranslationLanguage {
  ALL = "all",
  ENGLISH = "eng",
  RUSSIAN = "rus",
  UKRAINIAN = "ukr",
  POLISH = "pol",
  FRENCH = "fra",
  GERMAN = "ger",
  ITALIAN = "ita",
  SPANISH = "spa",
}

/** Every `ETranslationLanguage` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type TranslationLanguage = `${ETranslationLanguage}`;

/** How much one import run moved. */
export type TranslationParseCensus = {
  /** String tables read out of the scope. */
  filesRead: number;
  /** JSON sources created because nothing was there yet. */
  filesCreated: number;
  /** JSON sources rewritten because merging changed something in them. */
  filesUpdated: number;
  /** JSON sources left alone because merging changed nothing. */
  filesUnchanged: number;
  /** Files skipped without being read, because they are not string tables or hold no entries. */
  filesSkipped: number;
  /** Entries read out of the XML, before any merging. */
  entriesRead: number;
  /** Ids this run introduced to their file. */
  entriesInserted: number;
  /** Placeholders this run replaced with text. */
  entriesFilled: number;
  /** Ids whose text already matched what was read. */
  entriesUnchanged: number;
  /**
   * Ids whose existing text differed from what was read.
   *
   * Kept unless the run was told to overwrite, in which case this counts what was replaced.
   */
  entriesConflicted: number;
  /** `null` placeholders added for languages a file carries but a record did not. */
  placeholdersAdded: number;
};

/**
 * What one import run read, changed, and objected to.
 *
 * The census counts and the findings answer different questions and are kept apart for that reason:
 * a count says how much moved, a finding says which file has a problem and why. `subject` on a
 * finding is what makes a per-file breakdown unnecessary.
 */
export type TranslationParseResult = {
  /**
   * Whether the run read every table or was stopped between them.
   *
   * Each source is written whole through a staged replace, so a stopped import leaves the ones already written
   * complete and the rest untouched.
   */
  outcome: JobOutcome;
  duration: number;
  /** The language this run filed every entry under. */
  language: string;
  /**
   * Whether the run computed its answer without writing it.
   *
   * A dry run's whole result is what it would have written, so the counts below mean "would" rather
   * than "did" whenever this is set.
   */
  isDryRun: boolean;
  census: TranslationParseCensus;
  /**
   * The findings, in the shape every other XRF command reports them in.
   *
   * Sealed by `finalize` once the run is over, so the crate's own result is
   * what both the CLI and the desktop app deposit — neither has to restate the shape, and they
   * cannot drift apart by restating it differently.
   */
  report: Report;
};

/** An opened translations root, whichever layout it turned out to have. */
export type TranslationProjectDescriptor = {
  mode: TranslationProjectMode;
  /** The roots this project was opened over, echoed back so a follow-up read addresses the same trees. */
  roots: XrayRoots;
  /** Logical prefix the string tables were read from. */
  prefix: string;
  /** Every language the root offers, in discovery order. */
  languages: Array<string>;
  /**
   * The code page each language is written in, which is what limits the characters it can hold.
   *
   * Taken from the files themselves in gamedata mode, so a language XRF has never heard of still
   * reports the encoding its own declaration claims.
   */
  encodings: { [key in string]: string };
  /**
   * Whether every file this project holds is loose, so an editing session could save all of it.
   *
   * One flag rather than a tree of them, so a surface can say up front that a project opened over an
   * installation is read-only. Which particular file refuses is answered by its source's absent
   * physical path.
   */
  isEditable: boolean;
  /** Files keyed by the logical name the layout groups them under. */
  files: { [key in string]: TranslationFile };
  findings: Array<TranslationFinding>;
};

/** Which layout a translations root is read with. */
export enum ETranslationProjectMode {
  /** XRF sources: multi-language JSON and language-suffixed XML side by side in one tree. */
  SOURCE = "source",
  /** Shipped gamedata: `text\<language>\*.xml`, where the directory carries the language. */
  GAMEDATA = "gamedata",
}

/** Every `ETranslationProjectMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type TranslationProjectMode = `${ETranslationProjectMode}`;

/**
 * Where one language's copy of a file was actually found.
 *
 * Two paths because they answer different questions. The logical path is the engine identity, which
 * is what the file is; the physical path is where it happens to sit on this machine, which exists
 * only when the winning mount is a loose directory. An archived winner has none, and that absence is
 * the write guard — bytes inside a `.db` volume cannot be edited in place.
 *
 * **The physical path is for showing, never for addressing.** It is portable-formatted, so it has
 * already lost any name that is not valid Unicode and any `\` a host treats as an ordinary character.
 * A save renders the edited bytes by logical path (`render_edits`) and commits them through the VFS,
 * which resolves where the file really is.
 */
export type TranslationSource = {
  logicalPath: string;
  physicalPath: string | null;
};

/** One translation's text, which is a single line or a run of them joined on build. */
export type TranslationVariant = string | Array<string>;

/** What one language is missing from one file. */
export type TranslationVerifyLanguageSummary = {
  /** The source this counts, as the project addresses it. */
  file: string;
  language: string;
  /** Ids the file holds, which is the same for every language of that file. */
  checked: number;
  /** Ids with no text for this language, counting an explicit `null` as missing. */
  missing: number;
};

/** What a completeness check found, per file and language. */
export type TranslationVerifyResult = {
  /** Whether the run checked every source or was stopped between them. */
  outcome: JobOutcome;
  duration: number;
  checkedTranslationsCount: number;
  missingTranslationsCount: number;
  /** Per file and language, in discovery order. */
  languages: Array<TranslationVerifyLanguageSummary>;
};
