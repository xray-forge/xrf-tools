import { PeopleIcon } from "@xrf/ui/icons/people";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const CHARACTERS_EXPLORER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse character profiles and the assets they reference",
    group: EApplicationGroupId.GAMEPLAY,
    icon: <PeopleIcon />,
    id: EApplicationId.CHARACTERS_EXPLORER,
    label: "Characters explorer",
    path: "/characters-explorer",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
