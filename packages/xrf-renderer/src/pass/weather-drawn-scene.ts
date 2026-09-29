import { Nullable } from "@xrf/types";
import { Scene } from "three/webgpu";

/** What the weather draws over the frame this frame, rain or a bolt. */
export interface IWeatherDrawnScene {
  /** What to draw, or null while there is nothing to see or its draws are still compiling. */
  readonly drawn: Nullable<Scene>;
}
