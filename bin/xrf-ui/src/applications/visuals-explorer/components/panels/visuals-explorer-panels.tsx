import { default as AnimationIcon } from "@mui/icons-material/Animation";

import { VisualMotionsPanel } from "@/applications/visuals-explorer/components/panels/VisualMotionsPanel";
import { IEditorPanel } from "@/core/shell/panel/context";
import { VISUAL_PANELS } from "@/core/visuals/components/panels/visual-panels";

/**
 * The explorer's own motions panel, which plays what it lists.
 *
 * Not one of the shared inspection panels: it reads `VisualMotionService`, which only this application binds, so the
 * sequencer would fault on a panel list carrying it.
 */
const MOTIONS_PANEL: IEditorPanel = {
  icon: <AnimationIcon />,
  id: "motions",
  label: "Motions",
  render: () => <VisualMotionsPanel />,
};

/**
 * What the explorer contributes to the panel stripe.
 *
 * Inserted where the shared motions panel used to sit rather than appended, so the stripe order a user learned does
 * not move, and after the header, which stays the panel a model opens on.
 */
export const VISUALS_EXPLORER_PANELS: Array<IEditorPanel> = VISUAL_PANELS.flatMap((panel: IEditorPanel) =>
  panel.id === "bones" ? [panel, MOTIONS_PANEL] : [panel]
);
