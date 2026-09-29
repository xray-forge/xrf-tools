import { Nullable } from "@xrf/types";

/**
 * A weather effect asked for, one object a request, so asking for the same one twice still sends it.
 */
export interface ILevelWeatherEffectRequest {
  /** The effect, or null to end the one playing. */
  name: Nullable<string>;
}
