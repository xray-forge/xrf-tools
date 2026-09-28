import { Vector3 } from "three/webgpu";

/** A shadowed light as the planner takes it, in world space; copied, so one object may ask for every light. */
export interface ILightShadowRequest {
  isSpot: boolean;
  position: Vector3;
  /** A spot's direction and up, square to each other. */
  direction: Vector3;
  up: Vector3;
  /** A spot's whole cone. */
  cone: number;
  range: number;
  near: number;
  /** `(mean + luminance) / 2` of its colour, which a brighter light earns a larger map by. */
  intensity: number;
  /** Metres from the camera to its spatial sphere's edge, none inside it. */
  distance: number;
  /** `1 - dot(camera forward, light direction) / 2`: a light facing the camera earns more. */
  duel: number;
}
