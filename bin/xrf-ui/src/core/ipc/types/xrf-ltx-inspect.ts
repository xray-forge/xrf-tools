// Auto-generated rust bindings. Do not edit it manually.

/** One finding, already placed at the file and line a person has to open. */
export type LtxAnchoredFinding = {
  kind: LtxFindingKind;
  /** Engine identity of the entry point whose resolution the finding was produced under. */
  entry: string;
  /**
   * Engine identity of the config to open, where one is known.
   *
   * `None` means nothing recorded which config declares the section, which a patch dialect answers for a section
   * created by an override of something nothing declares.
   */
  file: string | null;
  /** One-based line in `file`, where the anchor reached one. */
  line: number | null;
  section: string | null;
  field: string | null;
  message: string;
  /** What the engine does with the same input, where the dialect said so. */
  engineBehaviour: string | null;
};

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

/** What kind of thing went wrong, which is also what decides how it was anchored. */
export type LtxFindingKind =
  /** The file would not parse. Carries its own line, from the parser. */
  | "parse"
  /** A section broke the scheme it is bound to, or is bound to one nothing declares. */
  | "scheme"
  /** The dialect had something to say about the root that is not a failure. */
  | "dialect"
  /** An `#include` reached no file. */
  | "include";

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

/**
 * Something the dialect wanted said about a root, short of refusing it.
 *
 * The serializable mirror of [`LtxResolutionDiagnostic`], which the core intentionally does not carry a wire shape for.
 * No severity, because there is only one: everything the engine refuses to start on comes back as an error from the
 * resolve, so a diagnostic exists precisely where the game would say nothing.
 */
export type LtxResolvedDiagnostic = {
  section: string;
  /** Engine identity of the config the dialect blamed, where it named one. */
  file: string | null;
  message: string;
  /** What the engine does with the same input, where that differs from reporting it. */
  engineBehaviour: string | null;
};

/** One resolved field: what it says, and why it says that. */
export type LtxResolvedField = {
  key: string;
  value: string;
  origin: LtxResolvedFieldOrigin;
};

/** How one resolved field came to hold the value it holds. */
export type LtxResolvedFieldOrigin =
  /** Written in the body of the section that holds it. */
  | { kind: "declared"; file: string | null }
  /**
   * Copied in by inheritance from the section that writes it, which is the ultimate writer and not the parent named
   * in the header.
   */
  | { kind: "inherited"; section: string; file: string | null }
  /** Won a load-order contest under a dialect that ranks statements rather than reading them in order. */
  | { kind: "loaded"; file: string; depth: number; operation: string }
  /** The resolution carries no record for this field, because none was asked for. */
  | { kind: "unrecorded" };

/** Every section one root resolved to, named and counted but not carried. */
export type LtxResolvedIndex = {
  /** Engine identity of the entry point this resolution was produced from. */
  entry: string;
  /** How the dialect that produced it names itself. */
  dialect: string;
  /**
   * Sections in the order the dialect answers them, which is authored order under standard LTX and name order under
   * DLTX. Not re-sorted: that order is the engine's own output, not a presentation choice.
   */
  sections: Array<LtxResolvedIndexEntry>;
  diagnostics: Array<LtxResolvedDiagnostic>;
};

/** One resolved section as the index lists it. */
export type LtxResolvedIndexEntry = {
  name: string;
  /**
   * Parents the header declared, read back from the declaring config.
   *
   * Not from the resolution: flattening inheritance is what resolving does, so a resolved section no longer records
   * what it inherited from.
   */
  parents: Array<string>;
  fieldCount: number;
  /** Engine identity of the config whose header declared the section, where the dialect stamped one. */
  origin: string | null;
};

/** One resolved section with its fields and where each of them came from. */
export type LtxResolvedSection = {
  /** Engine identity of the entry point this section was resolved from, so a consumer can key a cache by it. */
  entry: string;
  name: string;
  /** Parents the header declared, read back from the declaring config. */
  parents: Array<string>;
  /** Engine identity of the config whose header declared the section, where the dialect stamped one. */
  origin: string | null;
  /** Fields in resolved order, which is written order with inherited ones folded in ahead of them. */
  fields: Array<LtxResolvedField>;
};

/** What a scheme declares about one field. */
export type LtxSchemeFieldDeclaration = {
  /** The type as the scheme spells it - `u32`, `enum:pistol,rifle`, `condlist`. */
  dataType: string;
  isArray: boolean;
  isOptional: boolean;
  /** Whether this is the scheme's catch-all `*` rather than a declaration naming the field. */
  isAny: boolean;
};

/** One row of a scheme report: what the scheme asks for, and what the section answers. */
export type LtxSchemeFieldReport = {
  name: string;
  /** What the scheme declares about it, absent for a field the section holds and no declaration covers. */
  declared: LtxSchemeFieldDeclaration | null;
  /** What the section resolves to, absent for a declared field the section does not hold. */
  resolved: LtxResolvedField | null;
};

/**
 * What judges one resolved section, and how the section measures against it.
 *
 * The scheme is read off the resolution rather than off the file, so a section that inherits its binding is judged by
 * the same rule the verifier judges it by - which is the whole reason a binding is worth showing: nothing in the text
 * of `[wpn_child]:wpn_base` says it is a weapon.
 */
export type LtxSectionSchemeReport = {
  /** Engine identity of the entry point whose resolution this was read from. */
  entry: string;
  section: string;
  /** The `$scheme` the resolved section carries, absent when it carries none. */
  scheme: string | null;
  /** Whether a scheme file declares that name. False is itself a finding, and the verifier reports it as one. */
  isDeclared: boolean;
  /** Whether the declaration refuses fields it does not name and demands the ones it does not mark optional. */
  isStrict: boolean;
  /** The section the binding is written in, absent when this section writes it itself. */
  inheritedFrom: string | null;
  /** Every field the scheme declares and every field the section holds, merged. */
  fields: Array<LtxSchemeFieldReport>;
};

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
