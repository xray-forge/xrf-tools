/// <reference types="node" />

import { Dirent, readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";

import { describe, expect, it } from "@jest/globals";

/** The package's source root, which `#/` maps to. */
const SOURCE: string = __dirname;

/** This test, whose patterns name everything it forbids. */
const SELF: string = "renderer-architecture.test.ts";

/**
 * Every area, in layers from the bottom: an area reads the layers below its own and nothing beside or above it.
 * The leaves read nothing of the package; the contract is what crosses the thread; `internals` alone reads three past
 * its types; the page's client and the worker's host sit on top, apart.
 */
const LAYERS: ReadonlyArray<ReadonlyArray<string>> = [
  ["dds", "frame", "sampling"],
  ["contract"],
  ["geometry", "input", "internals", "lighting", "timing"],
  ["camera", "device", "texture", "visibility"],
  ["uniforms", "weather"],
  ["shader"],
  ["material"],
  ["scene"],
  ["pass"],
  ["capture", "graph", "pick"],
  ["client", "host"],
];

/** Three's own shader code, ported word for word, which keeps three's idiom so it reads against the original. */
const PORTED: ReadonlyArray<string> = ["pass/antialias/smaa-stages.tsl.ts"];

/** Modules that build nodes without being shaders: uniforms, and the binding of textures to samplers. */
const NODE_BUILDERS: ReadonlyArray<string> = ["uniforms/", "texture/renderer-textures.ts"];

/** The one area that reads three past its types. */
const INTERNALS: string = "internals/";

/** A class, a variable, or a constant made with `new`, at a module's top. */
const STATEFUL_DECLARATION: RegExp = /^(export )?(class|let|var)\b|^(export )?const \w+(: [^=]+)? = new\b/m;

/** A constant at a module's top made by one of TSL's node factories, or by a `Fn` called on the spot. */
const MODULE_NODE: RegExp =
  /^(export )?const \w+(: [^=]+)? = (uniform|uniformArray|storage|texture|attribute|varying|float|int|uint|u?vec[234]|ivec[234]|mat[234])\(|^(export )?const \w+(: [^=]+)? = Fn\(.*\)\(\)/m;

/** A `Fn` whose body spans lines, called on the spot at a module's top. */
const MODULE_CALL: RegExp = /^\}\)\(/m;

/** Every specifier a module names: `from "..."`, a bare `import "..."` and a dynamic `import("...")`. */
const SPECIFIER: RegExp = /(?:\bfrom\s+|\bimport\s*\(?\s*)"([^"]+)"/g;

/** Where TSL comes from besides `three/tsl`: three's display nodes, and its node sources. */
const TSL_SPECIFIER: RegExp = /^three\/(tsl$|addons\/tsl\/|examples\/jsm\/tsl\/|src\/nodes\/)/;

/** The `TSL` namespace, which `three/webgpu` exports whole. */
const TSL_NAMESPACE: RegExp = /\bimport\s*\{[^}]*\bTSL\b[^}]*\}\s*from\s*"three\/webgpu"/;

/** A read of three past its types: a member three marks private with `_`, or a renderer's backend. */
const THREE_INTERNAL: RegExp = /\._[A-Za-z]|\[\s*"_[A-Za-z]|\.backend\b/;

/** An exported type's declaration: a class, interface, enum or type alias, and its name. */
const EXPORTED_TYPE: RegExp = /^export (?:declare )?(?:default )?(?:abstract )?(class|interface|enum|type) (\w+)/gm;

/** One source file, read once for every rule. */
interface ISourceFile {
  /** Relative to the root, with forward slashes. */
  readonly path: string;
  readonly isTest: boolean;
  readonly text: string;
  /** The text without its comments, which may name what the code must not do. */
  readonly code: string;
  readonly imports: ReadonlyArray<string>;
}

/** Every source file under a directory, tests included. */
function readSources(directory: string = SOURCE): Array<ISourceFile> {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry: Dirent) => {
    const path: string = join(directory, entry.name);

    if (entry.isDirectory()) {
      return readSources(path);
    }

    if (!entry.name.endsWith(".ts")) {
      return [];
    }

    const text: string = readFileSync(path, "utf8");

    return [
      {
        code: text.replace(/\/\*[\s\S]*?\*\//g, "").replace(/(^|\s)\/\/.*$/gm, "$1"),
        imports: [...text.matchAll(SPECIFIER)].map((match: RegExpExecArray) => match[1]),
        isTest: entry.name.endsWith(".test.ts"),
        path: relative(SOURCE, path).replaceAll("\\", "/"),
        text,
      },
    ];
  });
}

/** @returns The layer an area stands in, or -1 for one no layer names. */
function toLayer(area: string): number {
  return LAYERS.findIndex((layer: ReadonlyArray<string>) => layer.includes(area));
}

/**
 * @param source - A module.
 * @returns Whether it exports more types than one, other than a discriminated union with its kind enum.
 */
function hasTypesBeside(source: ISourceFile): boolean {
  const kinds: Array<string> = [...source.text.matchAll(EXPORTED_TYPE)].map((match: RegExpExecArray) => match[1]);

  if (kinds.length < 2) {
    return false;
  }

  const union: RegExp = /^export type \w+(<[^=]*>)? =[^;]*\|/m;

  return !(kinds.length === 2 && kinds.includes("enum") && kinds.includes("type") && union.test(source.text));
}

/**
 * @param name - An exported type's name.
 * @returns The file it is named for: its name past the `I`, `E` or `T` prefix, in kebab case, `GBuffer` as one word.
 */
function toTypeModuleName(name: string): string {
  return name
    .replace(/^[IET](?=[A-Z])/, "")
    .replace(/(?<!^)(?=[A-Z])/g, "-")
    .toLowerCase()
    .replace(/(^|-)g-buffer/g, "$1gbuffer");
}

/**
 * @param source - A module exporting one type, or a union with its kind enum.
 * @returns Whether no type it exports gives it its name.
 */
function isNamedApart(source: ISourceFile): boolean {
  const names: Array<string> = [...source.text.matchAll(EXPORTED_TYPE)].map((match: RegExpExecArray) => match[2]);
  const module: string = (source.path.split("/").pop() as string).replace(/(\.tsl)?\.ts$/, "");

  return names.length > 0 && !names.some((name: string) => toTypeModuleName(name) === module);
}

describe("the renderer's architecture", () => {
  const sources: Array<ISourceFile> = readSources();
  const modules: Array<ISourceFile> = sources.filter((source: ISourceFile) => !source.isTest);
  const shaders: Array<ISourceFile> = modules.filter((source: ISourceFile) => source.path.endsWith(".tsl.ts"));

  it("places every area in a layer", () => {
    const areas: Array<string> = readdirSync(SOURCE, { withFileTypes: true })
      .filter((entry: Dirent) => entry.isDirectory())
      .map((entry: Dirent) => entry.name);

    expect(areas.filter((area: string) => toLayer(area) < 0)).toEqual([]);
  });

  it("keeps every area on the layers below it", () => {
    const violations: Array<string> = modules.flatMap(({ path, imports }: ISourceFile) => {
      const area: string = path.split("/")[0];

      // The package's entry points, beside every area.
      if (!path.includes("/")) {
        return [];
      }

      return imports
        .map((specifier: string) => /^#\/([^/]+)\//.exec(specifier)?.[1] ?? null)
        .filter((imported: string | null): imported is string => imported !== null && imported !== area)
        .filter((imported: string) => toLayer(imported) >= toLayer(area))
        .map((imported: string) => `${path} -> ${imported}`);
    });

    expect(violations).toEqual([]);
  });

  it("writes shader code in `.tsl.ts` modules only", () => {
    const violations: Array<string> = modules
      .filter(
        ({ path, imports, code }: ISourceFile) =>
          (imports.some((specifier: string) => TSL_SPECIFIER.test(specifier)) || TSL_NAMESPACE.test(code)) &&
          !path.endsWith(".tsl.ts") &&
          !NODE_BUILDERS.some((builder: string) => path.startsWith(builder))
      )
      .map(({ path }: ISourceFile) => path);

    expect(violations).toEqual([]);
  });

  // A node made at the module's top is one instance every builder shares: a node is built by a function, and one a
  // build shares is a `Fn(...).once()`.
  it("keeps `.tsl.ts` modules free of state: they build nodes and hold nothing", () => {
    const violations: Array<string> = shaders
      .filter(({ text }: ISourceFile) =>
        [STATEFUL_DECLARATION, MODULE_NODE, MODULE_CALL].some((pattern: RegExp) => pattern.test(text))
      )
      .map(({ path }: ISourceFile) => path);

    expect(violations).toEqual([]);
  });

  it("names every loop's counter, which three names `i` in every loop that nests", () => {
    const violations: Array<string> = shaders
      .filter(
        ({ path, text }: ISourceFile) =>
          path !== "shader/named-loop.tsl.ts" && !PORTED.includes(path) && /\bLoop\(/.test(text)
      )
      .map(({ path }: ISourceFile) => path);

    expect(violations).toEqual([]);
  });

  // Each read of three past its types is pinned by `three-internals.test.ts`; one made anywhere else is not.
  it("reads three past its types in `internals/` only, tests included", () => {
    const violations: Array<string> = sources
      .filter(
        ({ path, imports, code }: ISourceFile) =>
          !path.startsWith(INTERNALS) &&
          path !== SELF &&
          (imports.some((specifier: string) => specifier.startsWith("three/src/")) || THREE_INTERNAL.test(code))
      )
      .map(({ path }: ISourceFile) => path);

    expect(violations).toEqual([]);
  });

  it("exports one type a module, or a discriminated union with its kind enum", () => {
    const violations: Array<string> = modules
      .filter(({ path }: ISourceFile) => !/(^|\/|-)fixtures\.ts$/.test(path))
      .filter(hasTypesBeside)
      .map(({ path }: ISourceFile) => path);

    expect(violations).toEqual([]);
  });

  it("names a module holding a type for that type", () => {
    const violations: Array<string> = modules
      .filter(({ path }: ISourceFile) => !/(^|\/|-)fixtures\.ts$/.test(path))
      .filter(isNamedApart)
      .map(({ path }: ISourceFile) => path);

    expect(violations).toEqual([]);
  });
});
