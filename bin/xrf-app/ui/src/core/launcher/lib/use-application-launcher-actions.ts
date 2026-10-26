import { useCallback } from "react";

import { IApplicationDescriptor } from "@/core/routing/application";

export interface IApplicationLauncherActions {
  onWarm: () => void;
  onClick: () => void;
}

/**
 * Shares hover, focus, and activation behavior between launcher cards and rows.
 *
 * @param application - Application to preload and open.
 * @param onOpen - Opens the selected application.
 * @returns Handlers for warming and opening the application.
 */
export function useApplicationLauncherActions(
  application: IApplicationDescriptor,
  onOpen: (application: IApplicationDescriptor) => void
): IApplicationLauncherActions {
  const onWarm = useCallback((): void => {
    void application.preload?.();
  }, [application]);

  const onClick = useCallback((): void => {
    onOpen(application);
  }, [application, onOpen]);

  return { onWarm, onClick };
}
