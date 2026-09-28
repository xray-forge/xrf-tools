import { ERendererLightKind } from "#/contract/scene/renderer-light";
import { IRendererLightBase } from "#/contract/scene/renderer-light-base";

/** A light reaching every way around it. */
export interface IRendererPointLight extends IRendererLightBase {
  kind: ERendererLightKind.POINT;
}
