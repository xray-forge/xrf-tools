/// <reference types="node" />

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

import { describe, expect, it } from "@jest/globals";

/** The package's source root, which `#/` maps to. */
const SOURCE: string = resolve(__dirname, "..");

/** What only the page has: the client that owns the canvas, and the forwarder listening to the window. */
const PAGE_ONLY: ReadonlyArray<string> = ["client/", "input/render-input-forwarder.ts"];

/** Every package module a module imports, followed through `#/` and relative specifiers. */
function listReached(entry: string): Set<string> {
  const reached: Set<string> = new Set();
  const pending: Array<string> = [entry];

  while (pending.length) {
    const file: string = pending.pop() as string;

    if (reached.has(file)) {
      continue;
    }

    reached.add(file);

    for (const [, specifier] of readFileSync(file, "utf8").matchAll(/from\s+"([^"]+)"/g)) {
      if (specifier.startsWith("#/")) {
        pending.push(resolve(SOURCE, `${specifier.slice(2)}.ts`));
      } else if (specifier.startsWith(".")) {
        pending.push(resolve(dirname(file), `${specifier}.ts`));
      }
    }
  }

  return reached;
}

describe("the renderer worker", () => {
  it("reaches nothing of the page", () => {
    const reached: Array<string> = [...listReached(resolve(SOURCE, "host/renderer.worker.ts"))].map((file: string) =>
      file.slice(SOURCE.length + 1).replaceAll("\\", "/")
    );

    expect(reached.filter((file: string) => PAGE_ONLY.some((page: string) => file.startsWith(page)))).toEqual([]);
    expect(reached).toContain("host/renderer-host.ts");
  });
});
