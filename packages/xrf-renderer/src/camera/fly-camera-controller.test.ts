import { describe, expect, it, jest } from "@jest/globals";
import { Vector3 } from "three/webgpu";

import { FlyCameraController } from "#/camera/fly-camera-controller";
import { ERenderInput } from "#/contract/render-input";
import { IRenderInputEvent, toRenderInputEvent } from "#/contract/render-input-event";
import { ERendererCameraController } from "#/contract/renderer-camera";
import { IRendererFlyCamera } from "#/contract/renderer-fly-camera";
import { RenderProxyElement } from "#/input/render-proxy-element";

const DESCRIPTION: IRendererFlyCamera = {
  boost: 4,
  far: 1000,
  fieldOfView: 60,
  kind: ERendererCameraController.FLY,
  near: 0.1,
  position: [0, 10, 0],
  sensitivity: 0.003,
  speed: 1,
  target: [0, 10, -1],
};

function createController(target: [number, number, number]): {
  controller: FlyCameraController;
  element: RenderProxyElement;
} {
  const element: RenderProxyElement = new RenderProxyElement({ height: 1, pixelRatio: 1, width: 1 }, jest.fn());
  const controller: FlyCameraController = new FlyCameraController(element);

  controller.describe({ ...DESCRIPTION, target });

  return { controller, element };
}

function press(element: RenderProxyElement, code: string): void {
  element.dispatch({ ...toRenderInputEvent(ERenderInput.KEY_DOWN, new Event("keydown")), code });
}

function point(
  element: RenderProxyElement,
  type: ERenderInput,
  clientX: number,
  clientY: number,
  pointer: Partial<IRenderInputEvent> = {}
): void {
  element.dispatch({ ...toRenderInputEvent(type, new Event(type)), clientX, clientY, ...pointer });
}

function drag(element: RenderProxyElement, dx: number, dy: number): void {
  point(element, ERenderInput.POINTER_DOWN, 100, 100);
  point(element, ERenderInput.POINTER_MOVE, 100 + dx, 100 + dy);
  point(element, ERenderInput.POINTER_UP, 100 + dx, 100 + dy);
}

function toLook(controller: FlyCameraController): Vector3 {
  const { position, target } = controller.pose;

  return new Vector3(target[0] - position[0], target[1] - position[1], target[2] - position[2]);
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

  // Looking along -z, a drag to the right turns towards +x and a drag up looks up.
  it("turns the way it is dragged", () => {
    const { controller, element } = createController([0, 10, -1]);

    drag(element, 100, -50);
    controller.update(0);

    const look: Vector3 = toLook(controller);

    expect(look.x).toBeGreaterThan(0);
    expect(look.y).toBeGreaterThan(0);
    expect(look.z).toBeLessThan(0);
  });

  it("never pitches past straight up or down, so the horizon never flips", () => {
    const { controller, element } = createController([0, 10, -1]);

    drag(element, 0, -10_000);
    controller.update(0);

    const up: Vector3 = toLook(controller);

    drag(element, 0, 20_000);
    controller.update(0);

    const down: Vector3 = toLook(controller);

    expect(up.y).toBeLessThan(1);
    expect(up.y).toBeCloseTo(1, 5);
    expect(up.z).toBeLessThan(0);
    expect(down.y).toBeGreaterThan(-1);
    expect(down.y).toBeCloseTo(-1, 5);
    expect(down.z).toBeLessThan(0);
  });

  it("turns only by the main button of the first pointer, and follows that pointer alone", () => {
    const { controller, element } = createController([0, 10, -1]);

    point(element, ERenderInput.POINTER_DOWN, 0, 0, { button: 2 });
    point(element, ERenderInput.POINTER_MOVE, 100, 0, { button: 2 });
    point(element, ERenderInput.POINTER_DOWN, 0, 0, { isPrimary: false, pointerId: 2 });
    point(element, ERenderInput.POINTER_MOVE, 100, 0, { isPrimary: false, pointerId: 2 });
    controller.update(0);

    expect(toLook(controller).x).toBeCloseTo(0);

    point(element, ERenderInput.POINTER_DOWN, 0, 0, { pointerId: 1 });
    point(element, ERenderInput.POINTER_MOVE, 500, 0, { isPrimary: false, pointerId: 2 });
    point(element, ERenderInput.POINTER_UP, 500, 0, { isPrimary: false, pointerId: 2 });
    point(element, ERenderInput.POINTER_MOVE, 10, 0, { pointerId: 1 });
    controller.update(0);

    // Ten pixels at 0.003 radians each, of the first pointer's alone.
    expect(Math.atan2(toLook(controller).x, -toLook(controller).z)).toBeCloseTo(0.03);
  });

  it("moves no further in a frame than a quarter of a second takes it, after a stall", () => {
    const { controller, element } = createController([0, 10, -1]);

    press(element, "KeyW");
    controller.update(10);

    expect(controller.pose.position[2]).toBeCloseTo(-0.25);
  });

  it("lets go of every key and the drag once the canvas loses focus", () => {
    const { controller, element } = createController([0, 10, -1]);

    press(element, "KeyW");
    point(element, ERenderInput.POINTER_DOWN, 0, 0);
    element.dispatch(toRenderInputEvent(ERenderInput.BLUR, new Event(ERenderInput.BLUR)));
    point(element, ERenderInput.POINTER_MOVE, 100, 0);
    controller.update(1);

    expect(controller.pose.position).toEqual([0, 10, 0]);
    expect(toLook(controller).x).toBeCloseTo(0);
  });

  it("hears nothing once disposed, and gives the cursor back", () => {
    const { controller, element } = createController([0, 10, -1]);

    element.style.cursor = "auto";
    point(element, ERenderInput.POINTER_DOWN, 0, 0);
    controller.dispose();
    press(element, "KeyW");
    point(element, ERenderInput.POINTER_MOVE, 100, 0);
    controller.update(1);

    expect(element.style.cursor).toBe("auto");
    expect(controller.pose.position).toEqual([0, 10, 0]);
    expect(toLook(controller).x).toBeCloseTo(0);
  });

  it("keeps where it has flown when described again from the same start, and takes the new lens", () => {
    const { controller, element } = createController([0, 10, -1]);

    press(element, "KeyW");
    controller.update(0.1);

    expect(controller.describe({ ...DESCRIPTION, fieldOfView: 90, target: [0, 10, -1] })).toBe(false);
    expect(controller.pose.position[2]).toBeCloseTo(-0.1);
    expect(controller.camera.fov).toBe(90);
    expect(controller.describe({ ...DESCRIPTION, position: [5, 5, 5] })).toBe(true);
    expect(controller.pose.position).toEqual([5, 5, 5]);
  });
});
