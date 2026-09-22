import { readFileSync } from "fs";
import { dirname, resolve } from "path";

import { Nullable } from "@/lib/types/general";

/** Where `@/` points, read from this file rather than configured twice. */
export const SOURCE_ROOT: string = resolve(__dirname, "../..");

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

/**
 * Every module one module reaches, and the shortest chain each was reached through.
 *
 * @param entry - Absolute path of the module to walk from.
 * @param packages - Package names to record as reached, since they have no file of ours to resolve to.
 * @returns What it reaches, by module path or package name.
 */
export function listImportChains(
  entry: string,
  packages: ReadonlyArray<string> = []
): Map<string, ReadonlyArray<string>> {
  const reached: Map<string, ReadonlyArray<string>> = new Map();
  const pending: Array<[string, ReadonlyArray<string>]> = [[entry, [entry]]];

  while (pending.length) {
    const [current, path] = pending.shift() as [string, ReadonlyArray<string>];

    if (reached.has(current)) {
      continue;
    }

    reached.set(current, path);

    for (const [, specifier] of readFileSync(current, "utf8").matchAll(IMPORT)) {
      const resolved: Nullable<string> = resolveModule(specifier, current);

      if (resolved) {
        pending.push([resolved, [...path, resolved]]);
      } else if (packages.includes(specifier)) {
        reached.set(specifier, [...path, specifier]);
      }
    }
  }

  return reached;
}

/**
 * One chain as a person reads it: repository paths, oldest first.
 *
 * @param chain - What `listImportChains` recorded.
 * @returns The chain as one line, or an empty string when nothing was reached.
 */
export function formatImportChain(chain: ReadonlyArray<string> = []): string {
  return chain.map((it) => it.replace(SOURCE_ROOT, "").replaceAll("\\", "/")).join(" -> ");
}
