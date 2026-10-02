import { ELevelPanelId } from "@/core/level/lib/panels/level-panel-id";
import { ELevelPick } from "@/core/level/lib/pick/level-pick";
import { IPanelSetActiveCommand } from "@/core/shell/panel/panel-messages";

/** The panel each kind of pick is chosen in, and the side it is laid out on, which a click in the viewport opens. */
export const LEVEL_PICK_PANELS: Readonly<Record<ELevelPick, IPanelSetActiveCommand>> = {
  [ELevelPick.SPAWN]: { panelId: ELevelPanelId.SPAWN, side: "left" },
  [ELevelPick.SURFACE]: { panelId: ELevelPanelId.SURFACES, side: "left" },
};
