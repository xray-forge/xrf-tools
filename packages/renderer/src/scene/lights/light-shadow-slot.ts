import { Nullable } from "@xrf/types";

import { ILightShadowEntry } from "#/scene/lights/light-shadow-entry";

/** A light's entry shown, and the one drawn to replace it at another size while that is not yet whole. */
export interface ILightShadowSlot {
  shown: Nullable<ILightShadowEntry>;
  next: Nullable<ILightShadowEntry>;
}
