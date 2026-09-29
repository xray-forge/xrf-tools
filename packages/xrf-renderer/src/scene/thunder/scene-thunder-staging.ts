import { Scene } from "three/webgpu";

/**
 * The thunder's draws, built for a weather and waiting to compile before they draw.
 */
export interface ISceneThunderStaging {
  /** Its draws, every one shown, to compile against the frame the forward surfaces composite over. */
  scene: Scene;
  /** Compiled, or failed to: it draws from now on. */
  commit(): void;
  /** Never compiled: it waits to be taken again. */
  abandon(): void;
}
