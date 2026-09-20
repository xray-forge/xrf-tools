import { describe, expect, it } from "@jest/globals";
import { AxesHelper, Box3Helper, Group, Object3D } from "three";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { mockVisualBounds } from "@/fixtures/mocks/visual.mocks";
import { Nullable } from "@/lib/types/general";

import { DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG } from "./level-scene-config";
import { LevelPreviewFrame } from "./LevelPreviewFrame";

function options(overrides: Partial<ILevelViewOptions> = {}): ILevelViewOptions {
  return { ...DEFAULT_LEVEL_VIEW_OPTIONS, ...overrides };
}

function extentOf(parent: Object3D): Box3Helper {
  return parent.children.find((it): it is Box3Helper => it instanceof Box3Helper) as Box3Helper;
}

function axesOf(parent: Object3D): AxesHelper {
  return parent.children.find((it): it is AxesHelper => it instanceof AxesHelper) as AxesHelper;
}

function gridOf(parent: Object3D): Group {
  return parent.children.find((it): it is Group => it.type === "Group") as Group;
}

function framed(parent: Object3D): Array<Object3D> {
  return [...parent.children];
}

function level(): VisualBounds {
  return mockVisualBounds({ boundingBox: { max: { x: 600, y: 90, z: -100 }, min: { x: -400, y: -10, z: -600 } } });
}

describe("LevelPreviewFrame", () => {
  it("outlines the extent the level claims", () => {
    const parent: Group = new Group();
    const frame: LevelPreviewFrame = new LevelPreviewFrame(parent, DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG);

    frame.setBounds(level());
    frame.applyViewOptions(options());

    const extent: Box3Helper = extentOf(parent);

    expect(extent.visible).toBe(true);
    expect(extent.box.min.toArray()).toEqual([-400, -10, -600]);
    expect(extent.box.max.toArray()).toEqual([600, 90, -100]);
  });

  // The grid is centred on the origin while a level need not be anywhere near it, so sizing the grid to the level's
  // own width would leave the level off the edge of its own grid.
  it("reaches the level from the origin rather than spanning only the level", () => {
    const parent: Group = new Group();
    const frame: LevelPreviewFrame = new LevelPreviewFrame(parent, DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG);

    frame.setBounds(
      mockVisualBounds({ boundingBox: { max: { x: 2100, y: 10, z: 2100 }, min: { x: 2000, y: 0, z: 2000 } } })
    );

    // A hundred metres wide, two thousand from zero: the grid has to cover the gap, not the level.
    expect(frame.gridStep).toBeGreaterThanOrEqual(100);
  });

  it("measures in round steps, so the grid can be counted", () => {
    const parent: Group = new Group();
    const frame: LevelPreviewFrame = new LevelPreviewFrame(parent, DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG);

    frame.setBounds(level());

    expect([1, 2, 5].map((it) => it * 10 ** Math.floor(Math.log10(frame.gridStep)))).toContain(frame.gridStep);
  });

  // Three questions, three answers: a floor to judge scale against, a landmark, and what the level claims. Wanting
  // one is no reason to be shown the other two.
  it("switches the ground, the origin and the extent on their own", () => {
    const parent: Group = new Group();
    const frame: LevelPreviewFrame = new LevelPreviewFrame(parent, DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG);

    frame.setBounds(level());

    frame.applyViewOptions(options({ isAxesVisible: false, isBoundsVisible: false, isGridVisible: true }));

    expect(gridOf(parent).visible).toBe(true);
    expect(axesOf(parent).visible).toBe(false);
    expect(extentOf(parent).visible).toBe(false);

    frame.applyViewOptions(options({ isAxesVisible: true, isBoundsVisible: false, isGridVisible: false }));

    expect(gridOf(parent).visible).toBe(false);
    expect(axesOf(parent).visible).toBe(true);
    expect(extentOf(parent).visible).toBe(false);

    frame.applyViewOptions(options({ isAxesVisible: false, isBoundsVisible: true, isGridVisible: false }));

    expect(gridOf(parent).visible).toBe(false);
    expect(axesOf(parent).visible).toBe(false);
    expect(extentOf(parent).visible).toBe(true);
  });

  // A box around nothing is a dot at the origin, which reads as a level sitting there rather than as no measurement.
  it("draws no extent for a level that reports none, however the toggle stands", () => {
    const parent: Group = new Group();
    const frame: LevelPreviewFrame = new LevelPreviewFrame(parent, DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG);

    frame.setBounds(null);
    frame.applyViewOptions(options({ isGridVisible: true }));

    expect(extentOf(parent).visible).toBe(false);
  });

  it("takes everything it added back out of the scene", () => {
    const parent: Group = new Group();
    const frame: LevelPreviewFrame = new LevelPreviewFrame(parent, DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG);

    frame.setBounds(level());
    frame.dispose();

    expect(parent.children).toEqual([]);
  });

  it("holds a level opened after another one, rather than both", () => {
    const parent: Group = new Group();
    const frame: LevelPreviewFrame = new LevelPreviewFrame(parent, DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG);
    const before: Nullable<number> = framed(parent).length;

    frame.setBounds(level());
    frame.setBounds(mockVisualBounds({ boundingBox: { max: { x: 5, y: 5, z: 5 }, min: { x: -5, y: -5, z: -5 } } }));

    expect(framed(parent)).toHaveLength(before);
    expect(extentOf(parent).box.max.toArray()).toEqual([5, 5, 5]);
  });
});
