import { describe, expect, it, jest } from "@jest/globals";
import { Mesh, Scene } from "three/webgpu";

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

  it("lets a texture go only after the change before its release, merged into a later one, has applied", () => {
    const handler = createHandler();
    const queue: SceneChangeQueue<string> = new SceneChangeQueue(handler);
    const retired = jest.fn();

    queue.transact(() => queue.enlist("a"));
    queue.transact(() => {
      queue.releaseTexture("rock");
      queue.retire(retired);
    });
    // `a` again before its first change applied: that change now applies with this one, after the release.
    queue.transact(() => queue.enlist("a"));

    expect(handler.released).toEqual([]);
    expect(retired).not.toHaveBeenCalled();

    handler.ready.add("a");
    queue.advance();

    expect(handler.applied).toEqual(["a"]);
    expect(handler.released).toEqual(["rock"]);
    expect(retired).toHaveBeenCalledTimes(1);
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

  it("draws a released object's meshes until its release applies, then lets what it drew with go", () => {
    const handler = createHandler();
    const queue: SceneChangeQueue<string> = new SceneChangeQueue(handler);
    const scene: Scene = new Scene();
    const mesh: Mesh = new Mesh();
    const onDisposed = jest.fn();

    scene.add(mesh);
    queue.transact(() => queue.enlist("a"));
    queue.transact(() => {
      queue.withdraw("a", [mesh], onDisposed);
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

  it("lets a retired resource go with its change, only once every change before it has applied", () => {
    const handler = createHandler();
    const queue: SceneChangeQueue<string> = new SceneChangeQueue(handler);
    const release = jest.fn();

    queue.transact(() => queue.enlist("a"));
    handler.ready.add("b");
    queue.transact(() => {
      queue.retire(release);
      queue.enlist("b");
    });

    // `a` may still draw what `b`'s change retired.
    expect(handler.applied).toEqual([]);
    expect(release).not.toHaveBeenCalled();

    handler.ready.add("a");
    queue.advance();

    expect(handler.applied).toEqual(["a", "b"]);
    expect(release).toHaveBeenCalledTimes(1);
  });

  it("closes a transaction that threw with what it added, leaving the next one a change of its own", () => {
    const handler = createHandler();
    const queue: SceneChangeQueue<string> = new SceneChangeQueue(handler);

    expect(() =>
      queue.transact(() => {
        queue.enlist("a");
        throw new Error("refused");
      })
    ).toThrow("refused");

    handler.ready.add("b");
    queue.transact(() => queue.enlist("b"));

    expect(handler.applied).toEqual(["b"]);
    expect([...queue.pending]).toEqual(["a"]);
  });
});
