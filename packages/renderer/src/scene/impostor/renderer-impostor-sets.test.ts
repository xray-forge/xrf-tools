import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { Scene } from "three/webgpu";

import {
  IRendererImpostors,
  RENDERER_IMPOSTOR_CORNER_FLOATS,
  RENDERER_IMPOSTOR_CORNERS,
  RENDERER_IMPOSTOR_FACETS,
} from "#/contract/scene/renderer-impostors";
import { SceneChangeQueue } from "#/scene/change/scene-change-queue";
import { RendererImpostorSets } from "#/scene/impostor/renderer-impostor-sets";
import { StaticDraws } from "#/scene/static/static-draws";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";
import { StorageRetirement } from "#/uniforms/storage-retirement";

function createImpostors(count: number): IRendererImpostors {
  return {
    corners: new Float32Array(count * RENDERER_IMPOSTOR_CORNERS * RENDERER_IMPOSTOR_CORNER_FLOATS),
    factors: new Float32Array(count).fill(1),
    normals: new Float32Array(count * RENDERER_IMPOSTOR_FACETS * 4),
    spheres: new Float32Array(count * 4).fill(1),
  };
}

function createSets(): {
  draws: StaticDraws;
  sets: RendererImpostorSets;
  queue: SceneChangeQueue<string>;
  ready: Set<string>;
} {
  const draws: StaticDraws = new StaticDraws(
    new StaticDrawBuffers(new StorageRetirement(), { [EStaticPool.LODS]: 4 }),
    new Scene(),
    () => []
  );
  const ready: Set<string> = new Set();
  const queue: SceneChangeQueue<string> = new SceneChangeQueue({
    apply: () => {},
    canApply: (object: string) => ready.has(object),
    releaseTexture: () => {},
    settled: () => {},
  });
  const sets: RendererImpostorSets = new RendererImpostorSets(draws, (_: string, release: Nullable<() => void>) => {
    if (release) {
      queue.retire(release);
    }

    queue.enlist("clump");
  });

  return { draws, queue, ready, sets };
}

describe("RendererImpostorSets", () => {
  // An applied object's places name the run it wrote them with, which lasts while its rebuild waits to compile.
  it("keeps a set's run while the change rebuilding its users waits, and frees it once that change applies", () => {
    const { draws, queue, ready, sets } = createSets();

    queue.transact(() => sets.put("trees", createImpostors(2)));
    ready.add("clump");
    queue.advance();
    ready.delete("clump");
    queue.transact(() => sets.put("trees", createImpostors(2)));

    expect(sets.getStart("trees")).toBe(2);
    expect(draws.report.lods.used).toBe(4);

    queue.transact(() => sets.release("trees"));

    expect(draws.report.lods.used).toBe(4);

    ready.add("clump");
    queue.advance();

    expect(draws.report.lods.used).toBe(0);
    expect(sets.getStart("trees")).toBeNull();
    draws.dispose();
  });

  it("frees a replaced set's run with a change no object waits in at once", () => {
    const { draws, queue } = createSets();
    const lone: RendererImpostorSets = new RendererImpostorSets(draws, (_: string, release: Nullable<() => void>) => {
      if (release) {
        queue.retire(release);
      }
    });

    queue.transact(() => lone.put("rocks", createImpostors(1)));
    queue.transact(() => lone.put("rocks", createImpostors(1)));

    expect(draws.report.lods.used).toBe(1);
    expect(lone.getStart("rocks")).toBe(1);
    draws.dispose();
  });
});
