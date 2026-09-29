import { Scene } from "three/webgpu";

/**
 * The rain's draws, built for a weather and waiting to compile before they draw.
 */
export interface ISceneRainStaging {
  /** Its draws, to compile against the frame the forward surfaces composite over. */
  scene: Scene;
  /** Compiled, or failed to: it draws from now on. */
  commit(): void;
  /** Never compiled: it waits to be taken again. */
  abandon(): void;
}
