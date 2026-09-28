import { LIGHT_RECORD } from "#/scene/lights/light-record";

/** A vector of a light's record, as `LIGHT_RECORD` places it. */
export type TLightRecordVector = (typeof LIGHT_RECORD)[keyof typeof LIGHT_RECORD];
