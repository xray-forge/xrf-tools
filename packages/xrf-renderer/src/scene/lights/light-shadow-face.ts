import { Matrix4 } from "three/webgpu";

import { ILightShadowTile } from "#/scene/lights/light-shadow-tile";
import { EShadowCasterMotion } from "#/scene/static/shadow-caster-motion";
import { IShadowFrustum } from "#/visibility/shadow-frustum";

/** One face of a light's shadow: its view and projection, where it is drawn in the atlas, and whether it is current. */
export interface ILightShadowFace extends IShadowFrustum {
  /** Its camera's world matrix and its inverse, the view. */
  readonly world: Matrix4;
  readonly view: Matrix4;
  readonly projection: Matrix4;
  readonly near: number;
  readonly far: number;
  readonly tile: ILightShadowTile;
  /** Whether it was drawn since its square last lost what it held. */
  isDrawn: boolean;
  /** Whether something it casts from changed since it was drawn. */
  isStale: boolean;
  /** How the fastest caster standing in it moves, which has it drawn again while it does. */
  motion: EShadowCasterMotion;
  /** The frame it was last drawn in, which the faces over what moves take turns by. */
  drawnAt: number;
}
