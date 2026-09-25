import { uniform } from "three/tsl";

/**
 * What the temporal resolve reads besides the frame's targets: whether its history holds a frame to blend with.
 */
export class TemporalUniforms {
  /** One while the history holds the frame before, zero after a resize or on the first frame. */
  public readonly isHistoryValid = uniform(0);
  /** The least share of the current frame a pixel takes: its history is roughly this many frames' average, inverted. */
  public readonly currentWeight = uniform(0.1);
}
