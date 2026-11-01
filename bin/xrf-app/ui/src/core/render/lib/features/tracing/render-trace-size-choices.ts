import { ERenderTraceSize, RenderTraceSize } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

const RENDER_TRACE_SIZE_NAMES: Readonly<Record<RenderTraceSize, string>> = {
  [ERenderTraceSize.HALF]: "Half",
  [ERenderTraceSize.FULL]: "Full",
};

/**
 * @param size - A size a pass is traced at.
 * @returns Its name as the settings say it.
 */
export function describeRenderTraceSize(size: RenderTraceSize): string {
  return RENDER_TRACE_SIZE_NAMES[size];
}

/** The sizes a pass is traced at, cheapest first: half the frame each way, then the frame's own. */
export const RENDER_TRACE_SIZE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderTraceSize>> = Object.values(
  ERenderTraceSize
).map((value: RenderTraceSize) => ({ label: describeRenderTraceSize(value), value }));
