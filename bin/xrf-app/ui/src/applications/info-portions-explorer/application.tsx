import { default as HubIcon } from "@mui/icons-material/Hub";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const INFO_PORTIONS_EXPLORER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse info portions and what gives or requires them",
    group: EApplicationGroupId.GAMEPLAY,
    icon: <HubIcon />,
    id: EApplicationId.INFO_PORTIONS_EXPLORER,
    label: "Info portions explorer",
    path: "/info-portions-explorer",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
