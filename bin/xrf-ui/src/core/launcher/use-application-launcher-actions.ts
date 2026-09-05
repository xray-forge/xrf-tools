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
 * @param isEnabled - Whether the launcher entry accepts interaction.
 * @param onOpen - Opens the selected application.
 * @returns Handlers for warming and opening an enabled application.
 */
export function useApplicationLauncherActions(
  application: IApplicationDescriptor,
  isEnabled: boolean,
  onOpen: (application: IApplicationDescriptor) => void
): IApplicationLauncherActions {
  const onWarm = useCallback((): void => {
    if (isEnabled) {
      void application.preload?.();
    }
  }, [application, isEnabled]);

  const onClick = useCallback((): void => {
    if (isEnabled) {
      onOpen(application);
    }
  }, [application, isEnabled, onOpen]);

  return { onWarm, onClick };
}
