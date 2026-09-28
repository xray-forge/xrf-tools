import { describe, expect, it, jest } from "@jest/globals";

import { OrbitCameraController } from "#/camera/orbit-camera-controller";
import { ERendererCameraController } from "#/contract/renderer-camera";
import { ERendererCameraCommand } from "#/contract/renderer-camera-command";
import { IRendererOrbitCamera } from "#/contract/renderer-orbit-camera";
import { TRendererVector } from "#/contract/renderer-vector";
import { RenderProxyElement } from "#/input/render-proxy-element";

const START: IRendererOrbitCamera = {
  far: 100,
  fieldOfView: 45,
  kind: ERendererCameraController.ORBIT,
  near: 0.01,
  position: [0, 0, 10],
  target: [0, 0, 0],
};

function expectAt(actual: TRendererVector, expected: TRendererVector): void {
  actual.forEach((value: number, axis: number) => expect(value).toBeCloseTo(expected[axis]));
}

function createController(): OrbitCameraController {
  const controller: OrbitCameraController = new OrbitCameraController(
    new RenderProxyElement({ height: 1, pixelRatio: 1, width: 1 }, jest.fn())
  );

  controller.describe(START);

  return controller;
}

describe("OrbitCameraController", () => {
  it("keeps where it was moved when described again from the same start, and takes the new lens", () => {
    const controller: OrbitCameraController = createController();

    controller.command({ kind: ERendererCameraCommand.DOLLY, step: 0.5 });

    expect(controller.describe({ ...START, fieldOfView: 30 })).toBe(false);
    expect(controller.pose.position[2]).toBeCloseTo(5);
    expect(controller.camera.fov).toBe(30);
  });

  it("goes to a new start, and says it jumped", () => {
    const controller: OrbitCameraController = createController();

    controller.command({ kind: ERendererCameraCommand.DOLLY, step: 0.5 });

    expect(controller.describe({ ...START, target: [1, 0, 0] })).toBe(true);
    expectAt(controller.pose.position, [0, 0, 10]);
    expectAt(controller.pose.target, [1, 0, 0]);
  });

  it("takes the first description's start from its own", () => {
    const controller: OrbitCameraController = new OrbitCameraController(
      new RenderProxyElement({ height: 1, pixelRatio: 1, width: 1 }, jest.fn())
    );

    expectAt(controller.pose.position, [0, 0, 3]);
    expect(controller.describe(START)).toBe(true);
    expectAt(controller.pose.position, [0, 0, 10]);
  });
});
