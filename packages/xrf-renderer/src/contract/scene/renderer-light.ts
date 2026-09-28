import { IRendererPointLight } from "#/contract/scene/renderer-point-light";
import { IRendererSpotLight } from "#/contract/scene/renderer-spot-light";

/**
 * The shape a light reaches out in.
 */
export enum ERendererLightKind {
  /** Every way around it, to its range. */
  POINT = "point",
  /** Along its direction, within its cone, through its projector. */
  SPOT = "spot",
}

/** One local light, of either kind. */
export type TRendererLight = IRendererPointLight | IRendererSpotLight;
