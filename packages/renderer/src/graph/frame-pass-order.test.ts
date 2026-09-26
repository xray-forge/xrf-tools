import { describe, expect, it } from "@jest/globals";

import { IBaseFramePasses } from "#/graph/base-frame-passes";
import { IFrameOptionalPasses, toFramePassOrder } from "#/graph/frame-pass-order";
import { IRendererPass } from "#/pass/renderer-pass";

/** A pass that is nothing but its name. */
function toPass(name: string): IRendererPass {
  return { dispose: () => {}, name, render: () => {} };
}

const BASE: IBaseFramePasses = Object.fromEntries(
  ["cull", "gbuffer", "lateCull", "gbufferLate", "pyramid", "wallmarks", "sun", "combine", "forward", "overlay"].map(
    (name: string) => [name, toPass(name)]
  )
) as unknown as IBaseFramePasses;

const NONE: IFrameOptionalPasses = {
  ambientOcclusion: null,
  grass: null,
  lightShadows: null,
  lights: null,
  motionBackground: null,
  resolve: null,
  shadows: [],
  sharpen: null,
  smoothing: null,
  spatial: null,
};

/** The names of the frame's passes in order, with the optional ones given. */
function toOrder(optional: Partial<IFrameOptionalPasses>): Array<string> {
  return toFramePassOrder(BASE, { ...NONE, ...optional }, toPass("present")).map((pass: IRendererPass) => pass.name);
}

describe("the frame's pass order", () => {
  it("draws the base alone with every feature off", () => {
    expect(toOrder({})).toEqual([
      "cull",
      "gbuffer",
      "lateCull",
      "gbufferLate",
      "pyramid",
      "wallmarks",
      "sun",
      "combine",
      "forward",
      "overlay",
      "present",
    ]);
  });

  it("puts every stage of a resolved frame where it reads what it needs", () => {
    expect(
      toOrder({
        ambientOcclusion: toPass("ao"),
        grass: toPass("grass"),
        lightShadows: toPass("light-shadows"),
        lights: toPass("lights"),
        motionBackground: toPass("motion-background"),
        resolve: { ...toPass("fsr2"), beforeBlended: [toPass("fsr2-opaque")] },
        shadows: [toPass("shadow-0"), toPass("shadow-1")],
        sharpen: toPass("rcas"),
      })
    ).toEqual([
      "cull",
      "gbuffer",
      "grass",
      "lateCull",
      "gbufferLate",
      "motion-background",
      "pyramid",
      "wallmarks",
      "shadow-0",
      "shadow-1",
      "light-shadows",
      "sun",
      "lights",
      "ao",
      "combine",
      "fsr2-opaque",
      "forward",
      "fsr2",
      "rcas",
      "overlay",
      "present",
    ]);
  });

  it("smooths the helpers too, unless FSR 1 upscales what was smoothed before they draw", () => {
    expect(toOrder({ smoothing: toPass("antialias") }).slice(-3)).toEqual(["overlay", "antialias", "present"]);
    expect(
      toOrder({ sharpen: toPass("rcas"), smoothing: toPass("antialias"), spatial: toPass("fsr1") }).slice(-6)
    ).toEqual(["forward", "antialias", "fsr1", "rcas", "overlay", "present"]);
  });
});
