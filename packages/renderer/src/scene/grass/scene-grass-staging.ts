import { Scene } from "three/webgpu";

/** A grass build waiting to compile, which the renderer compiles off the frame in turn with the scene's. */
export interface ISceneGrassStaging {
  /** Its draws, to compile against the G-buffer. */
  scene: Scene;
  /** Compiled, or failed to: it plants and draws from now on. */
  commit(): void;
  /** Never compiled: it waits to be taken again. */
  abandon(): void;
}
