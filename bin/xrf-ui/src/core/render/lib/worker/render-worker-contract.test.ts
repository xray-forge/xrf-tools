import { resolve } from "path";

import { describe, expect, it } from "@jest/globals";

import { formatImportChain, listImportChains, SOURCE_ROOT } from "@/fixtures/utils/imports";

/** Every module that is the first line of a thread of its own. */
const WORKERS: ReadonlyArray<string> = [
  "core/level/lib/render/level-render.worker.ts",
  "core/visuals/lib/render/visual-preview.worker.ts",
];

/**
 * What a worker has none of.
 */
const PAGE_ONLY: ReadonlyArray<string> = [
  "lib/local-storage",
  "lib/dom",
  "lib/tauri",
  "core/ipc/invoke.ts",
  "core/ipc/raw.ts",
  "core/ipc/metrics",
  "core/render/lib/frame/dom-render-target.ts",
];

/**
 * The one rule that keeps a worker able to start.
 */
describe("the worker contract", () => {
  it.each(WORKERS)("reaches nothing of the page, from %s", (relative: string) => {
    const chains: Map<string, ReadonlyArray<string>> = listImportChains(resolve(SOURCE_ROOT, relative));

    const reached: Array<string> = [...chains.entries()]
      .filter(([module]) => PAGE_ONLY.some((it) => module.replaceAll("\\", "/").includes(`/src/${it}`)))
      .map(([, chain]) => formatImportChain(chain));

    expect(reached).toEqual([]);
  });
});
