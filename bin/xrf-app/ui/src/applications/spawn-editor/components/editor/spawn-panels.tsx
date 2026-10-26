import { default as InfoIcon } from "@mui/icons-material/Info";

import { IEditorPanel } from "@/core/shell/editor-shell";

import { SpawnRowDetailsPanel } from "./details/SpawnRowDetailsPanel";

/** What the spawn editor contributes to the panel stripe. */
export const SPAWN_EDITOR_PANELS: Array<IEditorPanel> = [
  {
    id: "details",
    label: "Row details",
    icon: <InfoIcon />,
    isOpenByDefault: false,
    render: () => <SpawnRowDetailsPanel />,
  },
];
