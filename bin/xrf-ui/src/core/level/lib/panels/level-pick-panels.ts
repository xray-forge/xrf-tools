import { ELevelPanelId } from "@/core/level/lib/panels/level-panel-id";
import { ELevelPick } from "@/core/level/lib/pick/level-pick";

/** The panel each kind of pick is chosen in, which a click in the viewport opens. */
export const LEVEL_PICK_PANELS: Readonly<Record<ELevelPick, ELevelPanelId>> = {
  [ELevelPick.SPAWN]: ELevelPanelId.SPAWN,
  [ELevelPick.SURFACE]: ELevelPanelId.SURFACES,
};
