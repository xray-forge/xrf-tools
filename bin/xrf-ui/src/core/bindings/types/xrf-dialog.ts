// Auto-generated rust bindings. Do not edit it manually.

import { XrayRoots } from "@/core/bindings/types/xrf-vfs";

/**
 * One whole dialog: its own elements and every phrase it declares.
 *
 * What a selection fetches, against the summary the project index already gave. Both names it was
 * addressed by are echoed back, so a response arriving late cannot be read as another dialog's — the
 * same rule the asset commands follow.
 */
export type DialogDescriptor = {
  /** Logical path of the file holding it, as the project keys that file. */
  logicalPath: string;
  id: string;
  /** Selection priority, negative for a dialog meant to sort last. */
  priority: number | null;
  /** Dialog-level elements — preconditions, info gates, `init_func` — excluding the phrases. */
  elements: Array<DialogElementDescriptor>;
  /** The language the phrase text was resolved in, echoed back. */
  language: string | null;
  /**
   * Phrases in document order.
   *
   * Empty is legitimate: `dm_traveler_dialog` carries only a precondition and an init function and
   * builds its phrases from script at runtime.
   */
  phrases: Array<DialogPhraseDescriptor>;
};

/** One child element of a dialog or a phrase, as written. */
export type DialogElementDescriptor = {
  /** The element name as written, such as `give_info`. */
  name: string;
  /** What that name means to the engine. */
  kind: DialogElementKind;
  /** Text content, with entity references already resolved. */
  value: string;
};

/**
 * What a dialog or phrase child element means to the engine.
 *
 * Classification only. The element keeps the name it was written with, so an element this does not
 * recognise still survives a round trip; mods add their own, and one shipped project uses a
 * `go_back` phrase element the engine never defined.
 */
export type DialogElementKind =
  /** Translation key of the line, not the line itself. */
  | "text"
  /** Script call producing the line at runtime, in place of a translation key. */
  | "scriptText"
  /** Script call run when the phrase is selected. */
  | "action"
  /** Script predicate gating visibility. */
  | "precondition"
  /** Id of a phrase that may follow this one. */
  | "next"
  /** Info portion granted. */
  | "giveInfo"
  /** Info portion revoked. */
  | "disableInfo"
  /** Info portion required. */
  | "hasInfo"
  /** Info portion that must be absent. */
  | "dontHasInfo"
  /** Whether selecting the phrase ends the conversation. */
  | "isFinal"
  /** Script call run when the dialog is initialised. */
  | "initFunc"
  /** Recognised container rather than a value: `phrase_list` or `phrase`. */
  | "container"
  /** Not part of the schema. Preserved and reported. */
  | "unknown";

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
  dialogs: Array<DialogSummaryDescriptor>;
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

/** One line of a conversation, as a canvas draws it. */
export type DialogPhraseDescriptor = {
  /** Unique within its dialog, and what `next` references. The entry phrase is `0`. */
  id: string;
  /**
   * Translation key of the line, which is what the file holds.
   *
   * Absent for a phrase whose line comes from `script_text`, which is a state and not a defect:
   * Anomaly does it 107 times.
   */
  textKey: string | null;
  /** The line itself, in the language this dialog was described for. */
  text: string | null;
  /** Whether selecting this phrase ends the conversation. */
  isFinal: boolean;
  /**
   * Whether the phrase sits inside a `phrase_list` rather than directly under its dialog.
   *
   * Both forms occur, and a later insertion has to reproduce the one the file already uses.
   */
  isInPhraseList: boolean;
  /** Ids that may follow this one, **in the order the player is offered them**. */
  next: Array<string>;
  /** Every child element in document order, including the ones projected above. */
  elements: Array<DialogElementDescriptor>;
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
  /** Languages the text tree offers, which is what a language switcher is built from. */
  languages: Array<string>;
  /** Distinct translation keys the text tree defines. */
  textKeys: number;
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

/**
 * One dialog, as the project index lists it.
 *
 * Named for the reduction it is. A descriptor carrying a domain type's own name mirrors that type —
 * [`DialogDescriptor`] mirrors `Dialog` — so a summary of one has to say so.
 */
export type DialogSummaryDescriptor = {
  id: string;
  phrases: number;
  priority: number | null;
};
