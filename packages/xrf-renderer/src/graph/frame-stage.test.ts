import { describe, expect, it } from "@jest/globals";

import { FrameStage } from "#/graph/frame-stage";

describe("a frame stage", () => {
  it("keeps its value while the key stands, and makes it again for another", () => {
    const released: Array<string> = [];
    const stage: FrameStage<string> = new FrameStage((value: string) => released.push(value));

    expect(stage.reconcile("smaa", (mode: string) => `pass:${mode}`)).toBe(true);
    expect(stage.reconcile("smaa", (mode: string) => `other:${mode}`)).toBe(false);
    expect(stage.value).toBe("pass:smaa");
    expect(stage.reconcile("fxaa", (mode: string) => `pass:${mode}`)).toBe(true);
    expect(stage.value).toBe("pass:fxaa");
    expect(released).toEqual(["pass:smaa"]);
  });

  it("lets its value go once nothing is wanted, and counts every making and letting go", () => {
    const released: Array<number> = [];
    const stage: FrameStage<number> = new FrameStage((value: number) => released.push(value));

    stage.reconcile(
      { count: 2 },
      ({ count }) => count,
      ({ count }) => `${count}`
    );
    stage.reconcile(
      { count: 2 },
      () => 99,
      ({ count }) => `${count}`
    );
    stage.reconcile(null, () => 0);

    expect(stage.value).toBeNull();
    expect(stage.generation).toBe(2);
    expect(released).toEqual([2]);
  });

  it("lets its value go for good once disposed", () => {
    const released: Array<boolean> = [];
    const stage: FrameStage<boolean> = new FrameStage((value: boolean) => released.push(value));

    stage.reconcile(true, () => true);
    stage.dispose();

    expect(stage.value).toBeNull();
    expect(released).toEqual([true]);
  });
});
