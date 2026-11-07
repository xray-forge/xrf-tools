import { AnimationIcon } from "@xrf/ui/icons/animation";
import { DataObjectIcon } from "@xrf/ui/icons/data-object";

import { IShellPanel } from "@/core/shell/panels/shell-panel";
import { VISUAL_PANELS } from "@/core/visuals/components/panels/visual-panels";
import { VisualMotionsPanel } from "@/tools/visuals-explorer/components/panels/VisualMotionsPanel";
import { VisualObjectSourcePanel } from "@/tools/visuals-explorer/components/panels/VisualObjectSourcePanel";

/**
 * The explorer's own motions panel, which plays what it lists.
 *
 * Not one of the shared inspection panels: it reads `VisualMotionService`, which only this tool binds, so the
 * sequencer would fault on a panel list carrying it.
 */
const MOTIONS_PANEL: IShellPanel = {
  icon: <AnimationIcon />,
  id: "motions",
  label: "Motions",
  render: () => <VisualMotionsPanel />,
};

/**
 * What an SDK object holds beyond its model; the explorer's own, as it alone opens objects.
 */
const SOURCE_PANEL: IShellPanel = {
  icon: <DataObjectIcon />,
  id: "source",
  label: "Source",
  render: () => <VisualObjectSourcePanel />,
};

/**
 * What the explorer contributes to the panel stripe.
 *
 * Inserted where the shared motions panel used to sit rather than appended, so the stripe order a user learned does
 * not move, and after the header, which stays the panel a model opens on; the object's source last.
 */
export const VISUALS_EXPLORER_PANELS: Array<IShellPanel> = [
  ...VISUAL_PANELS.flatMap((panel: IShellPanel) => (panel.id === "bones" ? [panel, MOTIONS_PANEL] : [panel])),
  SOURCE_PANEL,
];
