import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera, Vector3 } from "three";

import { LevelFlyCamera } from "@/core/level/lib/camera/level-fly-camera";

import { LevelFlyControls } from "./LevelFlyControls";

/** A canvas the controls can take focus on and receive events from, as the viewport's own is. */
function createElement(): HTMLElement {
  const element: HTMLElement = document.createElement("canvas");

  element.tabIndex = 0;
  element.setPointerCapture = () => undefined;

  document.body.appendChild(element);

  return element;
}

describe("LevelFlyControls", () => {
  it("drives the camera it was given rather than one of its own", () => {
    const fly: LevelFlyCamera = new LevelFlyCamera();
    const camera: PerspectiveCamera = new PerspectiveCamera();
    const controls: LevelFlyControls = new LevelFlyControls(fly, createElement());

    fly.lookAt(camera, new Vector3(0, 0, -10));
    controls.update(camera, 1);

    expect(camera.quaternion.equals(new PerspectiveCamera().quaternion)).toBe(true);

    controls.dispose();
  });

  // React's strict mode mounts, unmounts and mounts again. The camera's own state has to survive that, or the view
  // snaps back to facing down negative z the moment the viewport remounts.
  it("leaves where the camera is looking behind when it is disposed", () => {
    const fly: LevelFlyCamera = new LevelFlyCamera();
    const camera: PerspectiveCamera = new PerspectiveCamera();
    const element: HTMLElement = createElement();

    const first: LevelFlyControls = new LevelFlyControls(fly, element);

    fly.look(400, 0);
    first.update(camera, 0);

    const aimed: number = camera.quaternion.y;

    first.dispose();

    const second: LevelFlyControls = new LevelFlyControls(fly, element);

    second.update(camera, 0);

    expect(camera.quaternion.y).toBe(aimed);

    second.dispose();
  });

  it("moves the camera while a key it binds is held", () => {
    const fly: LevelFlyCamera = new LevelFlyCamera();
    const camera: PerspectiveCamera = new PerspectiveCamera();
    const element: HTMLElement = createElement();
    const controls: LevelFlyControls = new LevelFlyControls(fly, element);

    element.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyW" }));

    expect(controls.update(camera, 1)).toBe(true);
    expect(camera.position.z).toBeLessThan(0);

    element.dispatchEvent(new KeyboardEvent("keyup", { code: "KeyW" }));

    expect(controls.update(camera, 1)).toBe(false);

    controls.dispose();
  });

  // A viewport that loses focus holds no key, or the camera flies away unattended behind a dialog.
  it("drops every held key when the viewport loses focus", () => {
    const fly: LevelFlyCamera = new LevelFlyCamera();
    const camera: PerspectiveCamera = new PerspectiveCamera();
    const element: HTMLElement = createElement();
    const controls: LevelFlyControls = new LevelFlyControls(fly, element);

    element.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyW" }));
    element.dispatchEvent(new FocusEvent("blur"));

    expect(controls.update(camera, 1)).toBe(false);

    controls.dispose();
  });

  it("stops listening once disposed", () => {
    const fly: LevelFlyCamera = new LevelFlyCamera();
    const camera: PerspectiveCamera = new PerspectiveCamera();
    const element: HTMLElement = createElement();
    const controls: LevelFlyControls = new LevelFlyControls(fly, element);

    controls.dispose();
    element.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyW" }));

    expect(controls.update(camera, 1)).toBe(false);
  });
});
