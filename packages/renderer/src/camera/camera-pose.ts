import { Vector3 } from "three/webgpu";

import { IRendererCameraPose } from "#/contract/renderer-camera-pose";

/**
 * @param position - Where a camera stands.
 * @param target - The point it looks at.
 * @returns The two, as a report states them.
 */
export function toCameraPose(position: Vector3, target: Vector3): IRendererCameraPose {
  return { position: [position.x, position.y, position.z], target: [target.x, target.y, target.z] };
}

/**
 * @param a - A camera's start, as its description names it.
 * @param b - Another.
 * @returns Whether the two start in the same place looking at the same point.
 */
export function isSameCameraStart(a: IRendererCameraPose, b: IRendererCameraPose): boolean {
  return (
    a.position[0] === b.position[0] &&
    a.position[1] === b.position[1] &&
    a.position[2] === b.position[2] &&
    a.target[0] === b.target[0] &&
    a.target[1] === b.target[1] &&
    a.target[2] === b.target[2]
  );
}
