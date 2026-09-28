/// <reference types="node" />

import { readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";

import { describe, expect, it } from "@jest/globals";

/** The package's source root, which `#/` maps to. */
const SOURCE: string = __dirname;

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
  ["uniforms"],
  ["shader"],
  ["material"],
  ["scene"],
  ["pass"],
  ["capture", "graph"],
  ["client", "host"],
];

/** Three's own shader code, ported word for word, which keeps three's idiom so it reads against the original. */
const PORTED: ReadonlyArray<string> = ["pass/antialias/smaa-stages.tsl.ts"];

/** Modules that build nodes without being shaders: uniforms, and the binding of textures to samplers. */
const NODE_BUILDERS: ReadonlyArray<string> = ["uniforms/", "texture/renderer-textures.ts"];

/** Every source module but the tests, relative to the root, with forward slashes. */
function listModules(directory: string = SOURCE): Array<string> {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path: string = join(directory, entry.name);

    if (entry.isDirectory()) {
      return listModules(path);
    }

    return entry.name.endsWith(".ts") && !entry.name.endsWith(".test.ts")
      ? [relative(SOURCE, path).replaceAll("\\", "/")]
      : [];
  });
}

/** Every specifier a module imports from. */
function listImports(module: string): Array<string> {
  return [...readFileSync(join(SOURCE, module), "utf8").matchAll(/from\s+"([^"]+)"/g)].map(
    ([, specifier]) => specifier
  );
}

/** @returns The layer an area stands in, or -1 for one no layer names. */
function toLayer(area: string): number {
  return LAYERS.findIndex((layer: ReadonlyArray<string>) => layer.includes(area));
}

describe("the renderer's architecture", () => {
  const modules: Array<string> = listModules();
  const shaders: Array<string> = modules.filter((module: string) => module.endsWith(".tsl.ts"));

  it("places every area in a layer", () => {
    const areas: Array<string> = readdirSync(SOURCE, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name);

    expect(areas.filter((area: string) => toLayer(area) < 0)).toEqual([]);
  });

  it("keeps every area on the layers below it", () => {
    const violations: Array<string> = modules.flatMap((module: string) => {
      const area: string = module.split("/")[0];

      // The package's entry points, beside every area.
      if (!module.includes("/")) {
        return [];
      }

      return listImports(module)
        .map((specifier: string) => /^#\/([^/]+)\//.exec(specifier)?.[1] ?? null)
        .filter((imported: string | null): imported is string => imported !== null && imported !== area)
        .filter((imported: string) => toLayer(imported) >= toLayer(area))
        .map((imported: string) => `${module} -> ${imported}`);
    });

    expect(violations).toEqual([]);
  });

  it("writes shader code in `.tsl.ts` modules only", () => {
    const violations: Array<string> = modules.filter(
      (module: string) =>
        listImports(module).includes("three/tsl") &&
        !module.endsWith(".tsl.ts") &&
        !NODE_BUILDERS.some((builder: string) => module.startsWith(builder))
    );

    expect(violations).toEqual([]);
  });

  it("keeps `.tsl.ts` modules free of state: they build nodes and hold nothing", () => {
    const violations: Array<string> = shaders.filter((module: string) =>
      /^(export )?(class|let|var)\b|^(export )?const \w+(: [^=]+)? = new\b/m.test(
        readFileSync(join(SOURCE, module), "utf8")
      )
    );

    expect(violations).toEqual([]);
  });

  it("names every loop's counter, which three names `i` in every loop that nests", () => {
    const violations: Array<string> = shaders.filter(
      (module: string) =>
        module !== "shader/named-loop.tsl.ts" &&
        !PORTED.includes(module) &&
        /\bLoop\(/.test(readFileSync(join(SOURCE, module), "utf8"))
    );

    expect(violations).toEqual([]);
  });
});
