import { InfoIcon } from "@xrf/ui/icons/info";

import { IShellPanel } from "@/core/shell/panels/shell-panel";

import { SpawnRowDetailsPanel } from "./details/SpawnRowDetailsPanel";

/** What the spawn editor contributes to the panel stripe. */
export const SPAWN_EDITOR_PANELS: Array<IShellPanel> = [
  {
    id: "details",
    label: "Row details",
    icon: <InfoIcon />,
    isOpenByDefault: false,
    render: () => <SpawnRowDetailsPanel />,
  },
];
