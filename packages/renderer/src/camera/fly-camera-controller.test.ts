import { describe, expect, it, jest } from "@jest/globals";

import { FlyCameraController } from "#/camera/fly-camera-controller";
import { ERendererCameraController } from "#/contract/renderer-camera";
import { ERenderInput, toRenderInputEvent } from "#/contract/renderer-input";
import { RenderProxyElement } from "#/input/render-proxy-element";

/** A camera ten metres up, looking along `-z` and pitched down by the target, flying a metre a second. */
function createController(target: [number, number, number]): {
  controller: FlyCameraController;
  element: RenderProxyElement;
} {
  const element: RenderProxyElement = new RenderProxyElement({ height: 1, pixelRatio: 1, width: 1 }, jest.fn());
  const controller: FlyCameraController = new FlyCameraController(element);

  controller.describe({
    boost: 4,
    far: 1000,
    fieldOfView: 60,
    kind: ERendererCameraController.FLY,
    near: 0.1,
    position: [0, 10, 0],
    sensitivity: 0.003,
    speed: 1,
    target,
  });

  return { controller, element };
}

function press(element: RenderProxyElement, code: string): void {
  element.dispatch({ ...toRenderInputEvent(ERenderInput.KEY_DOWN, new Event("keydown")), code });
}

describe("FlyCameraController", () => {
  it("rises along its own up, which leans ahead as it is pitched down", () => {
    const { controller, element } = createController([0, 9, -1]);

    press(element, "KeyE");
    controller.update(0.1);

    const [x, y, z] = controller.pose.position;

    // Pitched down by 45 degrees, its up points half ahead, along -z.
    expect(x).toBeCloseTo(0);
    expect(y - 10).toBeCloseTo(0.1 * Math.SQRT1_2);
    expect(z).toBeCloseTo(-0.1 * Math.SQRT1_2);
  });

  it("rises straight up while level", () => {
    const { controller, element } = createController([0, 10, -1]);

    press(element, "KeyQ");
    controller.update(0.2);

    expect(controller.pose.position[1]).toBeCloseTo(9.8);
    expect(controller.pose.position[2]).toBeCloseTo(0);
  });
});
