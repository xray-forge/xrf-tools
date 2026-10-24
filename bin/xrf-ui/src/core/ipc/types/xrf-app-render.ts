// Auto-generated rust bindings. Do not edit it manually.

import { JobOutcome } from "@/core/ipc/types/xrf-job";
import { RenderViewportEvent } from "@/core/ipc/types/xrf-renderer";
import { WorldViewportEvent } from "@/core/ipc/types/xrf-world";

/** How a level map's or cubemap's capture ended: written whole, or stopped when asked, writing nothing. */
export type CaptureOutcome = {
  outcome: JobOutcome;
  /** The picture's pixels across and down, as written: a map's, or a cubemap's cross. */
  width: number;
  height: number;
};

/** A point of a viewport to name what its level draws under. */
export type RenderPickRequest = {
  /** Css pixels from the viewport's left edge. */
  x: number | null;
  /** Css pixels from its top edge. */
  y: number | null;
  /** Whether glass is passed by, naming what stands behind it. */
  isThrough: boolean;
};

/** What a native viewport tells its page, from the renderer drawing it or the world it shows; each tells its own kinds. */
export type ViewportEvent = RenderViewportEvent | WorldViewportEvent;
