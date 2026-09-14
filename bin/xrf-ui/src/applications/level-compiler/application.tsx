import { default as BuildIcon } from "@mui/icons-material/Build";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const LEVEL_COMPILER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Compile editable level scenes into game-ready locations",
    group: EApplicationGroupId.LEVEL,
    icon: <BuildIcon />,
    id: EApplicationId.LEVEL_COMPILER,
    label: "Level compiler",
    path: "/level-compiler",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
