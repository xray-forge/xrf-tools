import { describe, expect, it } from "@jest/globals";

import { IBaseFramePasses } from "#/graph/base-frame-passes";
import { IFrameOptionalPasses } from "#/graph/frame-optional-passes";
import { toFramePassOrder } from "#/graph/frame-pass-order";
import { IOcclusionFramePasses } from "#/graph/occlusion-frame-passes";
import { IRendererPass } from "#/pass/renderer-pass";

/** A pass that is nothing but its name. */
function toPass(name: string): IRendererPass {
  return { dispose: () => {}, name, render: () => {} };
}

const BASE: IBaseFramePasses = Object.fromEntries(
  ["cull", "gbufferEarly", "gbuffer", "wallmarks", "rainCover", "sun", "combine", "forward", "rain", "overlay"].map(
    (name: string) => [name, toPass(name)]
  )
) as unknown as IBaseFramePasses;

const OCCLUSION: IOcclusionFramePasses = {
  gbufferLate: toPass("gbufferLate"),
  lateCull: toPass("lateCull"),
  pyramid: toPass("pyramid"),
};

const NONE: IFrameOptionalPasses = {
  ambientOcclusion: null,
  distortion: null,
  exposure: null,
  grass: null,
  lightShadows: null,
  lights: null,
  motionBackground: null,
  occlusion: null,
  resolve: null,
  shadows: [],
  sharpen: null,
  smoothing: null,
  spatial: null,
  water: null,
};

/** The names of the frame's passes in order, with the optional ones given. */
function toOrder(optional: Partial<IFrameOptionalPasses>): Array<string> {
  return toFramePassOrder(BASE, { ...NONE, ...optional }, toPass("present")).map((pass: IRendererPass) => pass.name);
}

describe("the frame's pass order", () => {
  it("draws the base alone with every feature off", () => {
    expect(toOrder({})).toEqual([
      "cull",
      "gbufferEarly",
      "gbuffer",
      "wallmarks",
      "rainCover",
      "sun",
      "combine",
      "forward",
      "rain",
      "overlay",
      "present",
    ]);
  });

  it("puts every stage of a resolved frame where it reads what it needs", () => {
    expect(
      toOrder({
        ambientOcclusion: toPass("ao"),
        distortion: toPass("distortion"),
        grass: toPass("grass"),
        lightShadows: toPass("light-shadows"),
        lights: toPass("lights"),
        motionBackground: toPass("motion-background"),
        occlusion: OCCLUSION,
        resolve: { ...toPass("fsr2"), beforeBlended: [toPass("fsr2-opaque")] },
        shadows: [toPass("shadow-0"), toPass("shadow-1")],
        sharpen: toPass("rcas"),
        water: toPass("water"),
      })
    ).toEqual([
      "cull",
      "gbufferEarly",
      "lateCull",
      "gbufferLate",
      "pyramid",
      "gbuffer",
      "grass",
      "motion-background",
      "wallmarks",
      "shadow-0",
      "shadow-1",
      "light-shadows",
      "rainCover",
      "sun",
      "lights",
      "ao",
      "combine",
      "fsr2-opaque",
      "water",
      "forward",
      "rain",
      "distortion",
      "fsr2",
      "rcas",
      "overlay",
      "present",
    ]);
  });

  // Drawn over a distortion target nothing cleared, it would move the whole frame.
  it("moves what the water distorts only in a frame the water draws in", () => {
    expect(toOrder({ distortion: toPass("distortion") })).not.toContain("distortion");
    expect(toOrder({ distortion: toPass("distortion"), water: toPass("water") })).toContain("distortion");
  });

  it("smooths the helpers too, unless FSR 1 upscales what was smoothed before they draw", () => {
    expect(toOrder({ smoothing: toPass("antialias") }).slice(-3)).toEqual(["overlay", "antialias", "present"]);
    expect(
      toOrder({ sharpen: toPass("rcas"), smoothing: toPass("antialias"), spatial: toPass("fsr1") }).slice(-6)
    ).toEqual(["rain", "antialias", "fsr1", "rcas", "overlay", "present"]);
  });

  // The plain draws and the grass move with no version the culls see: in the depth, a cluster they hid while the
  // camera stood still would stay culled after they moved.
  it("reduces the static draws' depth alone, both phases', before any plain draw or the grass", () => {
    expect(toOrder({ grass: toPass("grass"), occlusion: OCCLUSION }).slice(0, 8)).toEqual([
      "cull",
      "gbufferEarly",
      "lateCull",
      "gbufferLate",
      "pyramid",
      "gbuffer",
      "grass",
      "wallmarks",
    ]);
  });

  // Measured from what combine wrote, before the water and the blended surfaces add theirs, as `phase_luminance` is.
  it("measures the exposure right after combine", () => {
    const order: Array<string> = toOrder({ exposure: toPass("exposure") });

    expect(order.indexOf("exposure")).toBe(order.indexOf("combine") + 1);
  });
});
