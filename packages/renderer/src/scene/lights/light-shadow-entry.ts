import { Sphere } from "three/webgpu";

import { ILightShadowFace } from "#/scene/lights/light-shadow-face";

/** A shadowed light's faces, made for the size its light asked. */
export interface ILightShadowEntry {
  readonly faces: ReadonlyArray<ILightShadowFace>;
  /** The square its faces were asked at, and the one they were given, smaller where the atlas had no room. */
  readonly asked: number;
  readonly size: number;
  readonly near: number;
  readonly far: number;
  /** Everything it reaches, which a change must touch to touch any of its faces. */
  readonly sphere: Sphere;
  /** The frame its light was last asked for. */
  seen: number;
}
