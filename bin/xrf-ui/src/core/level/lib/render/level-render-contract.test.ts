import { readFileSync } from "fs";
import { dirname, resolve } from "path";

import { describe, expect, it } from "@jest/globals";

import { Nullable } from "@/lib/types/general";

/** Where `@/` points, read from this file rather than configured twice. */
const SOURCE_ROOT: string = resolve(__dirname, "../../../..");

/** Modules a renderer on another thread is reached through, which is what must stay free of the renderer. */
const CONTRACT: ReadonlyArray<string> = [
  "core/level/lib/render/level-renderer.ts",
  "core/level/lib/render/level-render-protocol.ts",
  "core/level/lib/render/level-render-bridge.ts",
];

const IMPORT = /^\s*(?:import|export)[^"']*from\s+["']([^"']+)["']/gm;

/** Where one import specifier actually lands, or null for a package rather than a file of ours. */
function resolveModule(specifier: string, from: string): Nullable<string> {
  const base: string = specifier.startsWith("@/")
    ? resolve(SOURCE_ROOT, specifier.slice(2))
    : specifier.startsWith(".")
      ? resolve(dirname(from), specifier)
      : "";

  if (!base) {
    return null;
  }

  for (const candidate of [`${base}.ts`, `${base}.tsx`, resolve(base, "index.ts")]) {
    try {
      readFileSync(candidate);

      return candidate;
    } catch {
      continue;
    }
  }

  return null;
}

function listImportChains(entry: string): Map<string, ReadonlyArray<string>> {
  const reached: Map<string, ReadonlyArray<string>> = new Map();
  const pending: Array<[string, ReadonlyArray<string>]> = [[entry, [entry]]];

  while (pending.length) {
    const [current, path] = pending.pop() as [string, ReadonlyArray<string>];

    if (reached.has(current)) {
      continue;
    }

    reached.set(current, path);

    for (const [, specifier] of readFileSync(current, "utf8").matchAll(IMPORT)) {
      const resolved: Nullable<string> = resolveModule(specifier, current);

      if (resolved) {
        pending.push([resolved, [...path, resolved]]);
      } else if (specifier === "three") {
        reached.set("three", [...path, "three"]);
      }
    }
  }

  return reached;
}

/**
 * The one rule that makes a renderer movable to another thread.
 */
describe("the level render contract", () => {
  it.each(CONTRACT)("reaches nothing that draws, from %s", (relative: string) => {
    const chains: Map<string, ReadonlyArray<string>> = listImportChains(resolve(SOURCE_ROOT, relative));
    const chain: ReadonlyArray<string> = chains.get("three") ?? [];

    expect(chain.map((it) => it.replace(SOURCE_ROOT, "").replaceAll("\\", "/")).join(" -> ")).toBe("");
  });
});
