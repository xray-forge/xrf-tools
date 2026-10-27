import { useInjection } from "@wirestate/react";
import { useCallback, useMemo } from "react";

import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelViewService } from "@/core/level/services/level-view/level-view.service";

/** One of the toolbar's switches, and how it is turned over. */
export interface ILevelViewToggle {
  isOn: boolean;
  toggle: () => void;
}

/**
 * @param option - The switch.
 * @returns Whether it is on, and how a click turns it over.
 */
export function useLevelViewToggle(option: keyof ILevelViewOptions): ILevelViewToggle {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const isOn: boolean = viewService.options[option];

  const toggle = useCallback(() => viewService.toggleOption(option), [viewService, option]);

  return useMemo(() => ({ isOn, toggle }), [isOn, toggle]);
}
