import { Texture } from "three/webgpu";

/** What the accumulation reads: this frame's prepared inputs at the render size, and the history at the display's. */
export interface IFsrAccumulateInputs {
  /** The frame as drawn, whose coverage the output keeps. */
  frame: Texture;
  /** YCoCg, and the depth clip in alpha. */
  prepared: Texture;
  /** The reactive and accumulation masks. */
  reactiveMasks: Texture;
  dilatedMotion: Texture;
  /** Whether each drawn texel is a thin feature to lock. */
  locks: Texture;
  /** The mean log luma a 32nd a side. */
  shadingLuma: Texture;
  /** The frame before, resolved: colour, and the temporal reactivity, negative where it was in motion. */
  history: Texture;
  /** Each pixel's lock: its lifetime remaining, and the shading luma it was locked at. */
  lockStatus: Texture;
  /** Each pixel's four last lumas. */
  lumaHistory: Texture;
}
