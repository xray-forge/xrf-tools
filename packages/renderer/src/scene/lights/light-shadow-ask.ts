import { ILightShadowRequest } from "#/scene/lights/light-shadow-request";

/** A request the planner took this frame, copied, with the size the engine wants its faces at. */
export interface ILightShadowAsk extends ILightShadowRequest {
  /** The light, as the scene's lights number it. */
  index: number;
  engineSize: number;
}
