// Auto-generated rust bindings. Do not edit it manually.

/**
 * One dialog, as the project index lists it.
 *
 * Enough to draw a tree and pick something to open, and deliberately not the phrases: 502 dialogs
 * of those is a payload nobody reads, so a dialog is fetched when it is selected.
 */
export type DialogDescriptor = {
  id: string;
  phrases: number;
  priority: number | null;
};

/** One dialog file the project holds. */
export type DialogFileDescriptor = {
  /** Absolute path on disk, which is what a later write has to reach. */
  path: string;
  /** The code page the file was decoded with, and the one a rewrite has to use. */
  encoding: string;
  dialogs: Array<DialogDescriptor>;
};

/**
 * Something worth reporting about a project that was opened anyway.
 *
 * The reader refuses nothing on content, so this is how an off-schema element or an unreadable file
 * reaches a caller. Phase 4's validation produces `xrf_report::Finding` instead; this is the
 * narrower thing a project open can say about itself.
 */
export type DialogFinding = {
  rule: string;
  subject: string | null;
  message: string;
};

/**
 * An opened dialog project.
 *
 * Both roots are echoed back rather than left for the caller to re-derive: the mode and any
 * overrides decided them, and a follow-up read that guessed differently would address another tree.
 */
export type DialogProjectDescriptor = {
  mode: DialogProjectMode;
  root: string;
  dialogsRoot: string;
  translationsRoot: string;
  /** Files keyed by their path relative to the dialogs root, in discovery order. */
  files: { [key in string]: DialogFileDescriptor };
  findings: Array<DialogFinding>;
};

/**
 * Which layout a dialog project is read with.
 *
 * `Gamedata` is the default, unlike `TranslationProjectMode`: dialog tooling is aimed at shipped
 * game data first, and the XRF sources are the opt-in. The two are otherwise the same distinction.
 */
export type DialogProjectMode =
  /** Shipped gamedata: dialog text sits in `configs/text/<language>`, one file per language. */
  | "gamedata"
  /** XRF sources: dialog text sits in `translations`, one JSON file carrying every language. */
  | "source";
