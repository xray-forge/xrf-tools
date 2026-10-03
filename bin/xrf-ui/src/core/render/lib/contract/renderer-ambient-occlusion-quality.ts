/**
 * How many directions and steps a pixel's horizons are searched in: XeGTAO's own quality presets.
 */
export enum ERendererAmbientOcclusionQuality {
  /** One direction, two steps each way. */
  LOW = "low",
  /** Two directions, two steps. */
  MEDIUM = "medium",
  /** Three directions, three steps, `Base`'s choice. */
  HIGH = "high",
  /** Six directions, three steps. */
  ULTRA = "ultra",
}
