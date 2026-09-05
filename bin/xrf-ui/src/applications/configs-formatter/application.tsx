import { default as FormatAlignLeftIcon } from "@mui/icons-material/FormatAlignLeft";

import { CONFIGS_FORMATTER_HELP } from "@/applications/configs-formatter/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const CONFIGS_FORMATTER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Check or format LTX configuration files",
    group: EApplicationGroupId.CONFIGS,
    help: CONFIGS_FORMATTER_HELP,
    icon: <FormatAlignLeftIcon />,
    id: EApplicationId.CONFIGS_FORMATTER,
    label: "Configs formatter",
    path: "/configs-formatter",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
