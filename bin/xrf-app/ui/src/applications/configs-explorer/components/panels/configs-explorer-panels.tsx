import { DescriptionIcon } from "@xrf/ui/icons/description";
import { FormatListBulletedIcon as ListIcon } from "@xrf/ui/icons/format-list-bulleted";
import { RuleIcon } from "@xrf/ui/icons/rule";
import { WarningAmberIcon as WarningIcon } from "@xrf/ui/icons/warning-amber";

import { ConfigsMenuPanel } from "@/applications/configs-explorer/components/panels/ConfigsMenuPanel";
import { ConfigsProblemsPanel } from "@/applications/configs-explorer/components/panels/ConfigsProblemsPanel";
import { ConfigsSchemePanel } from "@/applications/configs-explorer/components/panels/ConfigsSchemePanel";
import { ConfigsSectionsPanel } from "@/applications/configs-explorer/components/panels/ConfigsSectionsPanel";
import { IEditorPanel } from "@/core/shell/editor-shell";

/**
 * What the explorer contributes to the panel stripes.
 */
export const CONFIGS_EXPLORER_PANELS: Array<IEditorPanel> = [
  {
    icon: <DescriptionIcon />,
    id: "configs",
    isOpenByDefault: true,
    label: "Configs",
    render: () => <ConfigsMenuPanel />,
    side: "left",
  },
  {
    icon: <ListIcon />,
    id: "sections",
    isOpenByDefault: true,
    label: "Sections",
    render: () => <ConfigsSectionsPanel />,
  },
  {
    icon: <RuleIcon />,
    id: "scheme",
    label: "Scheme",
    render: () => <ConfigsSchemePanel />,
  },
  {
    icon: <WarningIcon />,
    id: "problems",
    label: "Problems",
    render: () => <ConfigsProblemsPanel />,
  },
];
