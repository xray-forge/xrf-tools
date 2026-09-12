import { default as UnarchiveIcon } from "@mui/icons-material/Unarchive";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { ARCHIVES_UNPACKER_HELP } from "./help";

export const ARCHIVES_UNPACKER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Extract game archives into a directory",
    group: EApplicationGroupId.ARCHIVES,
    help: ARCHIVES_UNPACKER_HELP,
    icon: <UnarchiveIcon />,
    id: EApplicationId.ARCHIVES_UNPACKER,
    label: "Archives unpacker",
    path: "/archives-unpacker",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
