// Auto-generated rust bindings. Do not edit it manually.

import { XrayRoots } from "@/core/bindings/types/xrf-vfs";

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

/**
 * One dialog file the project holds.
 *
 * Keyed by its logical path, so the key is the engine identity and the value says where that identity
 * was actually found.
 */
export type DialogFileDescriptor = {
  /** Host path when the winner is a loose file; absent when it comes out of an archive. */
  physicalPath: string | null;
  /** Whether an edit could write this file back. False for an archived winner. */
  isEditable: boolean;
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
 * Both prefixes are echoed back rather than left for the caller to re-derive: the mode and any
 * overrides decided them, and a follow-up read that guessed differently would address another tree.
 */
export type DialogProjectDescriptor = {
  mode: DialogProjectMode;
  /** The roots this project was opened over, echoed back so a follow-up read addresses the same trees. */
  roots: XrayRoots;
  /** Logical prefix the dialogs were read from. */
  dialogsPrefix: string;
  /** Logical prefix dialog text is read from. */
  translationsPrefix: string;
  /** Whether every file the project holds is loose, so an editing session could save all of it. */
  isEditable: boolean;
  /** Files keyed by their logical path, in logical-path order. */
  files: { [key in string]: DialogFileDescriptor };
  findings: Array<DialogFinding>;
};

/**
 * Which layout a dialog project is read with.
 *
 * `Gamedata` is the default, unlike `TranslationProjectMode`: dialog tooling is aimed at shipped
 * game data first, and the XRF sources are the opt-in. The two are otherwise the same distinction.
 *
 * The mode decides only where dialog *text* sits. Both layouts keep the dialogs themselves at the
 * same place, and both are logical prefixes rather than host paths, so an installation reads the
 * same way a loose tree does.
 */
export type DialogProjectMode =
  /** Shipped gamedata: dialog text sits in `configs\text\<language>`, one file per language. */
  | "gamedata"
  /** XRF sources: dialog text sits in `translations`, one JSON file carrying every language. */
  | "source";
