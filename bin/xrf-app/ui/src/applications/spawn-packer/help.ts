import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const SPAWN_PACKER_HELP: IApplicationHelp = {
  summary:
    "Build one packed `.spawn` file from an unpacked spawn directory. Use it after working on chunks exported " +
    "by the spawn editor or unpacker.",
  workflow: [
    "Choose the directory holding the unpacked spawn chunks as `Source`.",
    "Choose the `.spawn` file to write under `Output spawn`, then select `Pack`.",
    "Follow the progress and read the result. Open the output in the spawn editor to inspect it.",
  ],
  nuances: [
    "The source initially points at the spawn unpacker's default output directory; check it if you unpacked elsewhere.",
    "The suggested output is `all.spawn` under the packer's output directory. Both paths can be changed.",
    "Cancellation stops before writing. Once the final write has started, it finishes.",
  ],
  limitations: [
    "The source must hold unpacked spawn chunks; a gamedata directory or packed `.spawn` file is not the input.",
    "An existing output file is overwritten.",
  ],
  relatedTools: [EApplicationId.SPAWN_UNPACKER, EApplicationId.SPAWN_EDITOR],
};
