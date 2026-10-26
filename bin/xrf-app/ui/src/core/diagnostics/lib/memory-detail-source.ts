import { Nullable } from "@xrf/types";

import { IEditorStatusDetail } from "@/core/shell/editor-shell/editor-status-detail";

/**
 * One more row for the memory hover, after the processes, read each time the processes are.
 *
 * @returns The row, or `null` where the source has nothing to say.
 */
export type TMemoryDetailSource = () => Nullable<IEditorStatusDetail>;
