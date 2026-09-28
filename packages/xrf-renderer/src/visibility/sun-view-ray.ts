import { Vector3 } from "three/webgpu";

/** One edge of the view: where it starts and which way it runs, normalized. */
export interface ISunViewRay {
  readonly origin: Vector3;
  readonly direction: Vector3;
}
