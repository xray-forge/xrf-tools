import { describe, expect, it, jest } from "@jest/globals";
import { BufferGeometry, Mesh, Scene } from "three/webgpu";

import { ISceneChangeHandler } from "#/scene/change/scene-change-handler";
import { SceneChangeQueue } from "#/scene/change/scene-change-queue";

/** A handler whose objects apply once marked ready, recording what it applied. */
function createHandler(): ISceneChangeHandler<string> & {
  ready: Set<string>;
  applied: Array<string>;
  released: Array<string>;
} {
  const ready: Set<string> = new Set();
  const applied: Array<string> = [];
  const released: Array<string> = [];

  return {
    applied,
    apply: (object: string) => applied.push(object),
    canApply: (object: string) => ready.has(object),
    ready,
    released,
    releaseTexture: (key: string) => released.push(key),
    settled: () => {},
  };
}

describe("SceneChangeQueue", () => {
  it("applies a transaction whole, once every object in it can draw", () => {
    const handler = createHandler();
    const queue: SceneChangeQueue<string> = new SceneChangeQueue(handler);

    handler.ready.add("a");
    queue.transact(() => {
      queue.enlist("a");
      queue.enlist("b");
    });

    expect(handler.applied).toEqual([]);
    expect(queue.hasPending).toBe(true);

    handler.ready.add("b");
    queue.advance();

    expect(handler.applied).toEqual(["a", "b"]);
    expect(queue.hasPending).toBe(false);
  });

  it("lets a later change apply before an earlier one that cannot", () => {
    const handler = createHandler();
    const queue: SceneChangeQueue<string> = new SceneChangeQueue(handler);

    handler.ready.add("b");
    queue.transact(() => queue.enlist("a"));
    queue.transact(() => queue.enlist("b"));

    expect(handler.applied).toEqual(["b"]);
    expect([...queue.pending]).toEqual(["a"]);
  });

  it("merges a change into a later one touching an object it waits on", () => {
    const handler = createHandler();
    const queue: SceneChangeQueue<string> = new SceneChangeQueue(handler);

    queue.transact(() => {
      queue.enlist("a");
      queue.enlist("b");
    });
    handler.ready.add("b");
    handler.ready.add("c");
    queue.transact(() => {
      queue.enlist("b");
      queue.enlist("c");
    });

    // `c` came with `b`, and `b` with `a`: none draws before `a` can.
    expect(handler.applied).toEqual([]);

    handler.ready.add("a");
    queue.advance();

    expect(handler.applied.sort()).toEqual(["a", "b", "c"]);
  });

  it("settles only once the outermost transaction ends", () => {
    const handler = createHandler();
    const queue: SceneChangeQueue<string> = new SceneChangeQueue(handler);

    handler.ready.add("a");
    queue.transact(() => {
      queue.transact(() => queue.enlist("a"));

      expect(handler.applied).toEqual([]);
    });

    expect(handler.applied).toEqual(["a"]);
  });

  it("lets a texture go only once every change before its release has applied", () => {
    const handler = createHandler();
    const queue: SceneChangeQueue<string> = new SceneChangeQueue(handler);

    queue.transact(() => queue.enlist("a"));
    queue.transact(() => queue.releaseTexture("rock"));

    expect(handler.released).toEqual([]);

    handler.ready.add("a");
    queue.advance();

    expect(handler.released).toEqual(["rock"]);
  });

  it("keeps a texture put again before its release applied", () => {
    const handler = createHandler();
    const queue: SceneChangeQueue<string> = new SceneChangeQueue(handler);

    queue.transact(() => queue.enlist("a"));
    queue.transact(() => queue.releaseTexture("rock"));
    queue.transact(() => queue.keepTexture("rock"));
    handler.ready.add("a");
    queue.advance();

    expect(handler.released).toEqual([]);
  });

  it("applies a budget of objects a settle, and always at least one change", () => {
    const handler = createHandler();
    const queue: SceneChangeQueue<string> = new SceneChangeQueue(handler, 2);

    handler.ready.add("a");
    handler.ready.add("b");
    handler.ready.add("c");
    queue.transact(() => {
      queue.enlist("a");
      queue.enlist("b");
      queue.enlist("c");
    });

    expect(handler.applied).toEqual(["a", "b", "c"]);

    queue.transact(() => queue.enlist("d"));
    queue.transact(() => queue.enlist("e"));
    queue.transact(() => queue.enlist("f"));
    ["d", "e", "f"].forEach((object: string) => handler.ready.add(object));
    queue.advance();

    expect(handler.applied).toEqual(["a", "b", "c", "d", "e"]);

    queue.advance();

    expect(handler.applied).toEqual(["a", "b", "c", "d", "e", "f"]);
  });

  it("draws a released object's meshes until its release applies, then lets its geometry go", () => {
    const handler = createHandler();
    const queue: SceneChangeQueue<string> = new SceneChangeQueue(handler);
    const scene: Scene = new Scene();
    const mesh: Mesh = new Mesh();
    const geometry: BufferGeometry = new BufferGeometry();
    const onDisposed = jest.fn();

    geometry.addEventListener("dispose", onDisposed);
    scene.add(mesh);
    queue.transact(() => queue.enlist("a"));
    queue.transact(() => {
      queue.withdraw("a", [mesh], [geometry]);
      queue.enlist("b");
    });

    expect(mesh.parent).toBe(scene);
    expect(onDisposed).not.toHaveBeenCalled();

    handler.ready.add("b");
    queue.advance();

    expect(handler.applied).toEqual(["b"]);
    expect(mesh.parent).toBeNull();
    expect(onDisposed).toHaveBeenCalledTimes(1);
  });
});
