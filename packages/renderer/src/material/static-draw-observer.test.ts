import { describe, expect, it } from "@jest/globals";
import * as THREE from "three/webgpu";
import { NodeBuilder, NodeMaterialObserver } from "three/webgpu";

import { StaticDrawObserver } from "#/material/static-draw-observer";

/** Three's refresh types, which its types leave out and the observer answers in. */
const { FULL, NONE, SHARED } = (THREE as unknown as { RenderObjectRefreshType: Record<string, number> })
  .RenderObjectRefreshType;

/** What an observer is built from: a node material's builder, as far as the observer reads it. */
function createObserver(): StaticDrawObserver {
  const builder = { context: {}, material: { positionNode: { isNode: true } }, object: {} };

  return new StaticDrawObserver(builder as unknown as NodeBuilder);
}

describe("StaticDrawObserver", () => {
  it("refreshes fully where its bundle records, the shared groups once a render call on a replay, else nothing", () => {
    const observer: StaticDrawObserver = createObserver();
    const renderObject = { bundle: { version: 1 } };

    expect(observer.needsRefresh(renderObject, { renderId: 1 })).toBe(FULL);
    expect(observer.needsRefresh(renderObject, { renderId: 2 })).toBe(SHARED);
    expect(observer.needsRefresh(renderObject, { renderId: 2 })).toBe(NONE);

    renderObject.bundle.version = 2;

    expect(observer.needsRefresh(renderObject, { renderId: 2 })).toBe(FULL);
  });

  it("leaves a render object outside any bundle to three", () => {
    expect(createObserver().needsRefresh({ bundle: null }, { renderId: 1 })).toBe(FULL);
  });

  // The observer rests on three internals its types leave out: this fails where an upgrade moved them.
  it("finds three's refresh decision, its render id and the refresh types it answers in", () => {
    expect(typeof (NodeMaterialObserver.prototype as unknown as { needsRefresh: unknown }).needsRefresh).toBe(
      "function"
    );
    expect(createObserver().renderId).toBe(0);
    expect([NONE, SHARED, FULL]).toEqual([0, 1, 2]);
  });
});
