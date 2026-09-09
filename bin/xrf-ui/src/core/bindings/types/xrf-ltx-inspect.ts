// Auto-generated rust bindings. Do not edit it manually.

/**
 * One config as written, carrying only what reading the text cannot answer.
 *
 * Keys and values are intentionally absent: a highlighter already colours them from the line itself, and a config tree
 * holds hundreds of thousands of them. What travels is what needs the parser's truth - where a section begins, whether
 * its parents resolve, which scheme it ends up bound to, and where an include actually landed.
 */
export type LtxFileStructure = {
  /** Engine identity of the config this describes. */
  path: string;
  /**
   * Entry points whose resolution reaches this config, in project order.
   *
   * Empty for a config nothing includes and that is not itself an entry point, which under a patch dialect means an
   * attachment.
   */
  entryPoints: Array<string>;
  sections: Array<LtxStructureSection>;
  includes: Array<LtxStructureInclude>;
  /**
   * Why the file did not parse, when it did not.
   *
   * Present means `sections` and `includes` are empty because nothing could be read, not because the file holds
   * neither.
   */
  parseError: LtxStructureParseError | null;
};

/**
 * One config's text, one entry per line, numbered the way every other record here anchors.
 *
 * Lines rather than one string because every finding, every header and every include is addressed by line, and a
 * viewer that had to re-split the text would be a second place where "which line is this" is decided.
 */
export type LtxFileText = {
  /** Engine identity of the config these lines came from. */
  path: string;
  /** Every line of the file, index `n` holding line `n + 1`. */
  lines: Array<string>;
  /**
   * Whether the lines came back through the parser rather than as authored bytes.
   *
   * True is the normal answer and says one thing was lost: a line holding only whitespace comes back empty. It is
   * recorded rather than hidden because an editor writing these lines back has to know they are not byte-identical.
   * False means the file did not parse and the raw split was used instead.
   */
  isNormalized: boolean;
};

/**
 * Every config a project holds, and what each one is to the project.
 *
 * The tree a person navigates and the list `ltx list` prints are the same record: one place decides what an entry
 * point is, so a command and a viewer can never disagree about which files stand on their own.
 */
export type LtxInventory = {
  /** Configs sorted by engine identity, which is the order the project itself assembles them in. */
  files: Array<LtxInventoryFile>;
};

/** One config, and what the project makes of it. */
export type LtxInventoryFile = {
  /** Lower-case, backslash-separated engine identity. */
  path: string;
  /** The mount that supplied it, as that mount describes itself - a directory, or an archive volume set. */
  source: string;
  /** Whether a loose file backs it, which is what decides if an editor could ever write to it. */
  isPhysical: boolean;
  role: LtxInventoryRole;
};

/** What one config is to the project holding it. */
export type LtxInventoryRole =
  /** Nothing includes it, so it resolves on its own and is a unit a check or a view can be asked for. */
  | { kind: "entryPoint" }
  /** Reached only through another config's `#include`. */
  | {
      kind: "included";
      /** Configs whose `#include` names it, in project order. */
      by: Array<string>;
    }
  /** A scheme declaration. Not verified against schemes itself, which is why it outranks every other role here. */
  | { kind: "schemeFile" }
  /** A file that patches another config rather than standing on its own, as the dialect identified it. */
  | { kind: "attachment" };

/** One `#include`, and the configs it actually reached. */
export type LtxStructureInclude = {
  /** One-based line the statement was written on. */
  line: number;
  /** The path or wildcard mask as authored. */
  statement: string;
  /** Engine identities the statement resolved to. Empty means it reached nothing. */
  resolved: Array<string>;
};

/** One name after the `:` of a header, and whether the resolution holds it. */
export type LtxStructureParent = {
  name: string;
  /** Whether a section by this name exists in the resolution this file was read against. */
  resolves: boolean;
};

/** Why one config would not parse. */
export type LtxStructureParseError = {
  /** One-based line the parser stopped on. */
  line: number;
  /** One-based column the parser stopped on. */
  column: number;
  message: string;
};

/** The `$scheme` a resolved section ends up carrying, and whether the project declares it. */
export type LtxStructureScheme = {
  name: string;
  /** Whether a scheme file declares it. False is itself a finding, and the verifier reports it as one. */
  isDeclared: boolean;
};

/** One section header, and what resolving the file it belongs to made of it. */
export type LtxStructureSection = {
  /** One-based line the header was written on. */
  line: number;
  /** The name exactly as bracketed, padding included - `[ wpn_base ]` is not `[wpn_base]` and nothing inherits it. */
  name: string;
  /** The header's operation prefix, spelled as the engine spells it: empty, `!`, `@` or `!!`. */
  operation: string;
  parents: Array<LtxStructureParent>;
  /** The scheme the resolved section is bound to, directly or through a parent. */
  scheme: LtxStructureScheme | null;
};
