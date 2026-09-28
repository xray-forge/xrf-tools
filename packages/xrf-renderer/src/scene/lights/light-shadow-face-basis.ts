import { TRendererVector } from "#/contract/renderer-vector";

/** One face of a point light's shadow: where it looks, and which way is up in it. */
export interface ILightShadowFaceBasis {
  readonly direction: TRendererVector;
  readonly up: TRendererVector;
}
