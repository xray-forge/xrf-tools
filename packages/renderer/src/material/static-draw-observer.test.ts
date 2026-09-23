import { describe, expect, it } from "@jest/globals";
import { NodeBuilder } from "three/webgpu";

import { RenderObjectRefreshType } from "#/internals/render-object-refresh";
import { StaticDrawObserver } from "#/material/static-draw-observer";

const { FULL, NONE, SHARED } = RenderObjectRefreshType;

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
});
