import { SelectedLevelDescription } from "@xrf/ipc/types/xrf-app-levels";
import { RenderFrameReport } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";

/** What a part of the level's viewport reads of the service owning it, each as it stands when read. */
export interface ILevelRenderContext {
  /** The viewport the level is drawn in, or null while none is attached. */
  getViewport(): Nullable<NativeViewport>;
  /** The level open in it, or null for none. */
  getLevel(): Nullable<SelectedLevelDescription>;
  /** What the viewport's recent frames cost and what the GPU drawing them can do; empty while none is attached. */
  getFrame(): RenderFrameReport;
}
