import { resolve } from "path";

import { describe, expect, it } from "@jest/globals";

import { formatImportChain, listImportChains, SOURCE_ROOT } from "@/fixtures/utils/imports";

/** Modules a renderer on another thread is reached through, which is what must stay free of the renderer. */
const CONTRACT: ReadonlyArray<string> = [
  "core/level/lib/render/level-renderer.ts",
  "core/level/lib/render/level-render-protocol.ts",
  "core/level/lib/render/level-render-bridge.ts",
  "core/level/lib/render/level-render-messages.ts",
];

/**
 * The one rule that makes a renderer movable to another thread.
 */
describe("the level render contract", () => {
  it.each(CONTRACT)("reaches nothing that draws, from %s", (relative: string) => {
    const chains: Map<string, ReadonlyArray<string>> = listImportChains(resolve(SOURCE_ROOT, relative), ["three"]);

    expect(formatImportChain(chains.get("three"))).toBe("");
  });
});
