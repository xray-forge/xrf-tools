import { Scene } from "three/webgpu";

/** A build waiting to compile, which the renderer compiles off the frame in turn with the scene's. */
export interface ISceneBuildStaging {
  /** Its draws, to compile against the target they draw into. */
  scene: Scene;
  /** Compiled, or failed to: it draws from now on. */
  commit(): void;
  /** Never compiled: it waits to be taken again. */
  abandon(): void;
}
