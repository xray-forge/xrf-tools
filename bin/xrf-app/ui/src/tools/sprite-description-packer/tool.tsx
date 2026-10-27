import { Inventory2Icon } from "@xrf/ui/icons/inventory2";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

export const SPRITE_DESCRIPTION_PACKER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Build a description sprite from individual icons",
    group: EToolGroupId.SPRITES,
    icon: <Inventory2Icon />,
    id: EToolId.SPRITE_DESCRIPTION_PACKER,
    label: "Sprite description packer",
    path: "/sprite-description-packer",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
