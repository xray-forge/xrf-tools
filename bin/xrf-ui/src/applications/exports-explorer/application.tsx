import { default as SwapHorizIcon } from "@mui/icons-material/SwapHoriz";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const EXPORTS_EXPLORER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse TypeScript extern declarations in an XRF project",
    group: EApplicationGroupId.EXPORTS,
    icon: <SwapHorizIcon />,
    id: EApplicationId.EXPORTS_EXPLORER,
    label: "Exports explorer",
    path: "/exports-explorer",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
