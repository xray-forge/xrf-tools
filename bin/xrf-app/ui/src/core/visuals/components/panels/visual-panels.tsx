import { AccountTreeIcon } from "@xrf/ui/icons/account-tree";
import { InfoIcon } from "@xrf/ui/icons/info";
import { LayersIcon } from "@xrf/ui/icons/layers";

import { IEditorPanel } from "@/core/shell/editor-shell";
import { VisualBonesPanel } from "@/core/visuals/components/panels/VisualBonesPanel";
import { VisualHeaderPanel } from "@/core/visuals/components/panels/VisualHeaderPanel";
import { VisualMaterialsPanel } from "@/core/visuals/components/panels/VisualMaterialsPanel";

/**
 * The inspection panels, for any surface showing a visual.
 *
 * One list rather than one per tool: what a model contains does not depend on which tool opened it, and
 * a surface offering no bone controls simply gets a bone tree that does not select. Motions are not here: playing them
 * needs a service a tool binds for itself, so each surface contributes its own motions panel.
 */
export const VISUAL_PANELS: Array<IEditorPanel> = [
  { id: "header", label: "Header", icon: <InfoIcon />, render: () => <VisualHeaderPanel /> },
  { id: "bones", label: "Bones", icon: <AccountTreeIcon />, render: () => <VisualBonesPanel /> },
  { id: "materials", label: "Materials", icon: <LayersIcon />, render: () => <VisualMaterialsPanel /> },
];
