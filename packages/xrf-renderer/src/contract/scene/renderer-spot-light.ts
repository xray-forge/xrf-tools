import { TRendererVector } from "#/contract/renderer-vector";
import { ERendererLightKind } from "#/contract/scene/renderer-light";
import { IRendererLightBase } from "#/contract/scene/renderer-light-base";

/** A light reaching along its direction, within its cone, through its projector. */
export interface IRendererSpotLight extends IRendererLightBase {
  kind: ERendererLightKind.SPOT;
  direction: TRendererVector;
  /** What turns its projector about its direction. */
  right: TRendererVector;
  /** Its whole cone, in radians. */
  cone: number;
  /** Its projector: the key its texture was put under. Without one it lights its whole cone white. */
  projector?: string;
}
