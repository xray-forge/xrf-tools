import { Nullable } from "@xrf/types";

import { IShellStatusDetail } from "@/core/shell/status-bar/shell-status-detail";

/**
 * One more row for the memory hover, after the processes, read each time the processes are.
 *
 * @returns The row, or `null` where the source has nothing to say.
 */
export type TMemoryDetailSource = () => Nullable<IShellStatusDetail>;
