/// <reference types="node" />

import { readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";

import { describe, expect, it } from "@jest/globals";

/** The package's source root, which `#/` maps to. */
const SOURCE: string = __dirname;

/**
 * What each worker-side area may not import: every area sits on the ones below it, and none reaches up.
 * From the bottom: `uniforms`, `shader`, `material`, `scene`, `pass`, then `graph` and `capture`, and `host` over all.
 */
const FORBIDDEN: Record<string, ReadonlyArray<string>> = {
  capture: ["client", "graph", "host"],
  contract: ["capture", "device", "graph", "host", "material", "pass", "scene", "shader", "timing", "uniforms"],
  device: ["client", "host"],
  graph: ["capture", "client", "host"],
  material: ["capture", "client", "device", "graph", "host", "pass", "scene"],
  pass: ["capture", "client", "device", "graph", "host"],
  scene: ["capture", "client", "device", "graph", "host", "pass"],
  shader: ["capture", "client", "device", "graph", "host", "material", "pass", "scene"],
  timing: ["capture", "client", "device", "graph", "host"],
  uniforms: ["capture", "client", "device", "graph", "host", "material", "pass", "scene", "shader"],
};

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

describe("the renderer's architecture", () => {
  const modules: Array<string> = listModules();

  it("keeps every area on the ones below it", () => {
    const violations: Array<string> = modules.flatMap((module: string) => {
      const forbidden: ReadonlyArray<string> = FORBIDDEN[module.split("/")[0]] ?? [];

      return listImports(module)
        .filter((specifier: string) => forbidden.some((area: string) => specifier.startsWith(`#/${area}/`)))
        .map((specifier: string) => `${module} -> ${specifier}`);
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
    const violations: Array<string> = modules.filter(
      (module: string) =>
        module.endsWith(".tsl.ts") && /^export (class|let)\b/m.test(readFileSync(join(SOURCE, module), "utf8"))
    );

    expect(violations).toEqual([]);
  });
});
