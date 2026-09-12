import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const SPRITE_EQUIPMENT_PACKER_HELP: IApplicationHelp = {
  summary:
    "Build an equipment DDS sprite from individual icons using the inventory grid positions and sizes declared " +
    "by a system configuration.",
  workflow: [
    "Choose the directory of loose icons as `Source` and the `.dds` file to write as `Output`.",
    "Choose the `system.ltx` that names the icons. Enable `DLTX` when it must be read with DLTX patch rules.",
    "Select `Pack`, follow its progress, and inspect the result. Open the output in the equipment sprite editor " +
      "with the same configuration to check the layout.",
  ],
  nuances: [
    "For a section without a custom icon path, the packer looks for `<section>.png`, then `<section>.dds` in " +
      "the source directory.",
    "The configuration determines placement; the packer does not rearrange icons to find free space.",
    "Sections can share a grid position. If their artwork differs, only the last icon packed into that position survives.",
    "Cancellation stops before the output is written. Once writing starts, it finishes.",
  ],
  limitations: [
    "The output DDS is overwritten. The source icons and system configuration are the inputs to keep for later rebuilds.",
    "Packing does not update inventory grid coordinates in the configuration.",
  ],
  relatedTools: [EApplicationId.SPRITE_EQUIPMENT_EDITOR, EApplicationId.CONFIGS_EXPLORER],
};
