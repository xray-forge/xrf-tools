import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const SPRITE_EQUIPMENT_EDITOR_HELP: IApplicationHelp = {
  summary:
    "Inspect an equipment sprite alongside the configuration that maps inventory items to its cells. " +
    "When loose icons are available beside the sprite, rebuild it and inspect the result in the same view.",
  workflow: [
    "Choose the equipment `.dds` and the `system.ltx` that describes its icons.",
    "Enable `DLTX` if the configuration uses DLTX patch rules, then select `Open`.",
    "Use the grid and zoom controls to inspect the sheet. Select a cell to see the sections using it.",
    "After editing loose icons outside the application, use `Repack sprite` and confirm the displayed source and output.",
  ],
  nuances: [
    "Opening reads the sprite and configuration without writing either file.",
    "Repack looks for a sibling directory named after the sprite without its extension: " +
      "`ui_icon_equipment.dds` uses `ui_icon_equipment`. The action is unavailable when that source is absent.",
    "Repack uses the open project's system configuration and DLTX choice, then reloads the rebuilt sprite.",
    "Several sections can share one cell, so cell details may list more than one item.",
  ],
  limitations: [
    "The current view does not paint icons or edit their configuration coordinates. Make those changes in the source files.",
    "Repack overwrites the open DDS and has no undo. Use the equipment sprite packer to choose another output path.",
  ],
  relatedTools: [EApplicationId.SPRITE_EQUIPMENT_PACKER, EApplicationId.CONFIGS_EXPLORER],
};
