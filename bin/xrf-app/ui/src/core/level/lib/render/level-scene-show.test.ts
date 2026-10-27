import { beforeEach, describe, expect, it, jest } from "@jest/globals";

import { ILevelScene } from "@/core/level/lib/load/level-scene";
import { LevelSceneShow } from "@/core/level/lib/render/level-scene-show";
import { mockRenderLoadReport } from "@/core/render/fixtures/render.mocks";

describe("LevelSceneShow", () => {
  const onLeft = jest.fn<() => void>();
  const onResident = jest.fn<() => void>();

  function ready(scene: number, isReady: boolean = true): void {
    show.noteLoad(mockRenderLoadReport({ isReady, scene }));
  }

  let show: LevelSceneShow;

  beforeEach(() => {
    onLeft.mockClear();
    onResident.mockClear();
    show = new LevelSceneShow({ onLeft, onResident });
  });

  it("reveals a level opened once the scene it is drawn as is resident whole, and once", () => {
    expect(show.open("level", null)).toBe(true);
    expect(onLeft).toHaveBeenCalledTimes(1);

    // Ready before the renderer named the scene, which is still the level drawn before it.
    ready(3);
    expect(show.isRevealed).toBe(false);

    show.noteShown(4);
    ready(4, false);
    ready(3);
    expect(show.isRevealed).toBe(false);

    ready(4);
    ready(4);
    expect(show.isRevealed).toBe(true);
    expect(onResident).toHaveBeenCalledTimes(1);
  });

  it("reveals a level whose scene was resident before the renderer named it", () => {
    show.open("level", null);
    ready(4);
    show.noteShown(4);

    expect(show.isRevealed).toBe(true);
    expect(onResident).toHaveBeenCalledTimes(1);
  });

  it("reveals nothing while no level is open", () => {
    expect(show.open(null, null)).toBe(true);
    show.noteShown(null);
    ready(0);

    expect(show.isRevealed).toBe(false);
    expect(onResident).not.toHaveBeenCalled();
  });

  it("keeps a level read again in the session shown, and opens another session's anew", () => {
    show.open("level", null);
    show.noteShown(4);
    ready(4);

    expect(show.open("level", null)).toBe(false);
    expect(show.isRevealed).toBe(true);
    expect(onLeft).toHaveBeenCalledTimes(1);

    expect(show.open("other", null)).toBe(true);
    expect(show.isRevealed).toBe(false);
    expect(onLeft).toHaveBeenCalledTimes(2);
  });

  it("opens a session anew while its first scene is still unnamed", () => {
    show.open("level", null);

    expect(show.open("level", null)).toBe(true);
  });

  // The backend moves the scene onto a level its session streams afresh; the renderer names it in its reports.
  it("follows the level into the scene it streams as afresh, revealed once that scene is resident", () => {
    const opened: ILevelScene = { revision: 0, sessionId: "level" };

    show.open("level", opened);
    show.noteShown(4);
    ready(4);
    onLeft.mockClear();

    // The scene the level opened as, or another session's, is the opening's own.
    show.move(opened);
    show.move({ revision: 1, sessionId: "other" });
    expect(show.isRevealed).toBe(true);
    expect(onLeft).not.toHaveBeenCalled();

    show.move({ revision: 1, sessionId: "level" });
    expect(show.isRevealed).toBe(false);
    expect(onLeft).toHaveBeenCalledTimes(1);

    // The scene followed out of reports nothing of the one streamed afresh.
    ready(4);
    expect(show.isRevealed).toBe(false);

    ready(5);
    expect(show.isRevealed).toBe(true);
    expect(onResident).toHaveBeenCalledTimes(2);
  });

  // The renderer reports the scene it streams as from the moment the backend moved it, which can be before the page hears.
  it("reveals a level moved into a scene already reported resident", () => {
    show.open("level", null);
    show.noteShown(4);
    ready(4);
    ready(5);

    show.move({ revision: 1, sessionId: "level" });

    expect(show.isRevealed).toBe(true);
    expect(onResident).toHaveBeenCalledTimes(2);
  });

  it("forgets the level shown on release, which no later report reveals", () => {
    show.open("level", null);
    show.noteShown(4);
    ready(4);
    onLeft.mockClear();

    show.release();
    expect(show.isRevealed).toBe(false);
    expect(onLeft).toHaveBeenCalledTimes(1);

    ready(4);
    expect(show.isRevealed).toBe(false);
    expect(show.open("level", null)).toBe(true);
  });
});
