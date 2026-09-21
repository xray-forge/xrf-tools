import { describe, expect, it } from "@jest/globals";

import { ILevelFlyMotion } from "@/core/level/lib/camera/level-fly-motion";

import { LevelFlyControls } from "./LevelFlyControls";

function createElement(): HTMLElement {
  const element: HTMLElement = document.createElement("canvas");

  element.tabIndex = 0;
  element.setPointerCapture = () => undefined;

  document.body.appendChild(element);

  return element;
}

function pointer(type: string, fields: Record<string, number> = {}): Event {
  return Object.assign(new Event(type, { bubbles: true }), fields);
}

function look(element: HTMLElement, movementX: number, movementY: number): void {
  element.dispatchEvent(pointer("pointerdown", { pointerId: 1 }));
  element.dispatchEvent(pointer("pointermove", { movementX, movementY }));
}

describe("LevelFlyControls", () => {
  it("reports nothing before anything has happened", () => {
    const controls: LevelFlyControls = new LevelFlyControls(createElement());

    expect(controls.drain()).toEqual({ keys: expect.any(Object), lookX: 0, lookY: 0 });

    controls.dispose();
  });

  it("gathers a drag as movement to look by", () => {
    const element: HTMLElement = createElement();
    const controls: LevelFlyControls = new LevelFlyControls(element);

    look(element, 40, -10);

    const motion: ILevelFlyMotion = controls.drain();

    expect(motion.lookX).toBe(40);
    expect(motion.lookY).toBe(-10);

    controls.dispose();
  });

  // A look is a movement, and counting one twice turns the camera twice as far. The frame that takes it is the
  // one that applies it, so taking it is what forgets it.
  it("forgets a look once it has been drained", () => {
    const element: HTMLElement = createElement();
    const controls: LevelFlyControls = new LevelFlyControls(element);

    look(element, 40, 0);
    controls.drain();

    expect(controls.drain().lookX).toBe(0);

    controls.dispose();
  });

  it("gathers movement across a whole drag rather than only its last step", () => {
    const element: HTMLElement = createElement();
    const controls: LevelFlyControls = new LevelFlyControls(element);

    look(element, 10, 0);
    element.dispatchEvent(pointer("pointermove", { movementX: 15 }));

    expect(controls.drain().lookX).toBe(25);

    controls.dispose();
  });

  it("ignores movement while nothing is dragging", () => {
    const element: HTMLElement = createElement();
    const controls: LevelFlyControls = new LevelFlyControls(element);

    element.dispatchEvent(pointer("pointermove", { movementX: 40 }));

    expect(controls.drain().lookX).toBe(0);

    controls.dispose();
  });

  it("reports a key it binds as held while it is", () => {
    const element: HTMLElement = createElement();
    const controls: LevelFlyControls = new LevelFlyControls(element);

    element.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyW" }));

    expect(controls.drain().keys.forward).toBe(true);

    element.dispatchEvent(new KeyboardEvent("keyup", { code: "KeyW" }));

    expect(controls.drain().keys.forward).toBe(false);

    controls.dispose();
  });

  // Keys are held rather than gathered: draining says what is down now, and a key still down is still down.
  it("keeps reporting a key that is still held", () => {
    const element: HTMLElement = createElement();
    const controls: LevelFlyControls = new LevelFlyControls(element);

    element.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyW" }));
    controls.drain();

    expect(controls.drain().keys.forward).toBe(true);

    controls.dispose();
  });

  // A viewport that loses focus holds no key, or the camera flies away unattended behind a dialog.
  it("drops every held key and any half-finished look when the viewport loses focus", () => {
    const element: HTMLElement = createElement();
    const controls: LevelFlyControls = new LevelFlyControls(element);

    element.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyW" }));
    look(element, 40, 0);
    element.dispatchEvent(new FocusEvent("blur"));

    const motion: ILevelFlyMotion = controls.drain();

    expect(motion.keys.forward).toBe(false);
    expect(motion.lookX).toBe(0);

    controls.dispose();
  });

  // What a drain reports is a copy: a frame that read it and then held on to it would watch the keys change
  // under it as the next frame's are gathered.
  it("hands over a copy of what is held rather than what it goes on gathering into", () => {
    const element: HTMLElement = createElement();
    const controls: LevelFlyControls = new LevelFlyControls(element);

    const motion: ILevelFlyMotion = controls.drain();

    element.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyW" }));

    expect(motion.keys.forward).toBe(false);

    controls.dispose();
  });

  it("stops listening once disposed", () => {
    const element: HTMLElement = createElement();
    const controls: LevelFlyControls = new LevelFlyControls(element);

    controls.dispose();
    element.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyW" }));

    expect(controls.drain().keys.forward).toBe(false);
  });
});
