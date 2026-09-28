import { describe, expect, it } from "@jest/globals";

import { ERendererCameraController } from "#/contract/renderer-camera";
import { IRendererFlyCamera } from "#/contract/renderer-fly-camera";
import { RendererCameraRig } from "#/host/renderer-camera-rig";
import { RenderProxyElement } from "#/input/render-proxy-element";

const START: IRendererFlyCamera = {
  boost: 4,
  far: 1000,
  fieldOfView: 60,
  kind: ERendererCameraController.FLY,
  near: 0.2,
  position: [10, 20, 30],
  sensitivity: 0.003,
  speed: 10,
  target: [10, 20, 29],
};

function createRig(): RendererCameraRig {
  return new RendererCameraRig(new RenderProxyElement({ height: 1, pixelRatio: 1, width: 1 }, () => undefined));
}

describe("RendererCameraRig", () => {
  it("describes the camera while a cut is still pending, as a first description before any frame is", () => {
    const rig: RendererCameraRig = createRig();

    rig.describe(START);

    expect(rig.pose.position).toEqual([10, 20, 30]);
    expect(rig.takeCut()).toBe(true);
  });

  it("keeps the history for the same start with new speeds, and cuts for a new start", () => {
    const rig: RendererCameraRig = createRig();

    rig.describe(START);
    rig.takeCut();
    rig.describe({ ...START, speed: 20 });

    expect(rig.takeCut()).toBe(false);

    rig.describe({ ...START, position: [0, 0, 0] });

    expect(rig.pose.position).toEqual([0, 0, 0]);
    expect(rig.takeCut()).toBe(true);
  });
});
