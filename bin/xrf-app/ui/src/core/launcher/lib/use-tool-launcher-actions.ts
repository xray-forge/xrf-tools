import { useCallback } from "react";

import { IToolDescriptor } from "@/core/routing/tool";

export interface IToolLauncherActions {
  onWarm: () => void;
  onClick: () => void;
}

/**
 * Shares hover, focus, and activation behavior between launcher cards and rows.
 *
 * @param tool - Tool to preload and open.
 * @param onOpen - Opens the selected tool.
 * @returns Handlers for warming and opening the tool.
 */
export function useToolLauncherActions(
  tool: IToolDescriptor,
  onOpen: (tool: IToolDescriptor) => void
): IToolLauncherActions {
  const onWarm = useCallback((): void => {
    void tool.preload?.();
  }, [tool]);

  const onClick = useCallback((): void => {
    onOpen(tool);
  }, [tool, onOpen]);

  return { onWarm, onClick };
}
