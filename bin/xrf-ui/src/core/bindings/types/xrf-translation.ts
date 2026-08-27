// Auto-generated rust bindings. Do not edit it manually.

import { XrayRoots } from "@/core/bindings/types/xrf-vfs";

/** How much one import run moved. */
export type ProjectParseCensus = {
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

/** What one language is missing from one file. */
export type ProjectVerifyLanguageSummary = {
  /** The source this counts, as the project addresses it. */
  file: string;
  language: string;
  /** Ids the file holds, which is the same for every language of that file. */
  checked: number;
  /** Ids with no text for this language, counting an explicit `null` as missing. */
  missing: number;
};

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
export type TranslationProjectMode =
  /** XRF sources: multi-language JSON and language-suffixed XML side by side in one tree. */
  | "source"
  /** Shipped gamedata: `text\<language>\*.xml`, where the directory carries the language. */
  | "gamedata";

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
 * A write resolves the logical path through the VFS instead and asks the asset, which still holds the
 * real one — see `apply_edits_to_asset`.
 */
export type TranslationSource = {
  logicalPath: string;
  physicalPath: string | null;
};

/** One translation's text, which is a single line or a run of them joined on build. */
export type TranslationVariant = string | Array<string>;
