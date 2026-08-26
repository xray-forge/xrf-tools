import { default as AccountTreeIcon } from "@mui/icons-material/AccountTree";
import { default as AnimationIcon } from "@mui/icons-material/Animation";
import { default as InfoIcon } from "@mui/icons-material/Info";
import { default as LayersIcon } from "@mui/icons-material/Layers";

import { IEditorPanel } from "@/core/shell/panel/context";
import { VisualBonesPanel } from "@/core/visuals/components/panels/VisualBonesPanel";
import { VisualHeaderPanel } from "@/core/visuals/components/panels/VisualHeaderPanel";
import { VisualMaterialsPanel } from "@/core/visuals/components/panels/VisualMaterialsPanel";
import { VisualMotionsPanel } from "@/core/visuals/components/panels/VisualMotionsPanel";

/**
 * The inspection panels, for any surface showing a visual.
 *
 * One list rather than one per application: what a model contains does not depend on which application opened it, and
 * a surface offering no bone controls simply gets a bone tree that does not select.
 */
export const VISUAL_PANELS: Array<IEditorPanel> = [
  { id: "header", label: "Header", icon: <InfoIcon />, render: () => <VisualHeaderPanel /> },
  { id: "bones", label: "Bones", icon: <AccountTreeIcon />, render: () => <VisualBonesPanel /> },
  { id: "motions", label: "Motions", icon: <AnimationIcon />, render: () => <VisualMotionsPanel /> },
  { id: "materials", label: "Materials", icon: <LayersIcon />, render: () => <VisualMaterialsPanel /> },
];
