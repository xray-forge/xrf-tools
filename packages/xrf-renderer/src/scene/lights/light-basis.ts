import { Vector3 } from "three/webgpu";

/** A light's frame in world space: where it stands, and a spot's direction with its right and up square to it. */
export interface ILightBasis {
  readonly position: Vector3;
  readonly direction: Vector3;
  readonly right: Vector3;
  readonly up: Vector3;
}
