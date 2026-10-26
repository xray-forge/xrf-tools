import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const SPAWN_UNPACKER_HELP: IApplicationHelp = {
  summary:
    "Extract a packed `.spawn` file into a directory of chunks for inspection or editing outside the application. " +
    "Use the spawn packer to turn those chunks back into a packed file.",
  workflow: [
    "Choose the packed `.spawn` file as `Source`.",
    "Choose a `Destination` directory and select `Unpack`.",
    "Follow the progress and read the result before working with the extracted chunks.",
  ],
  nuances: [
    "The destination starts with the unpacker's default output directory and can be changed.",
    "Cancellation stops before writing. A write already in progress finishes.",
  ],
  limitations: [
    "Files with matching names in the destination are replaced. Use a separate directory for each spawn you unpack.",
    "This extracts spawn chunks, not archive contents or an entire gamedata tree.",
  ],
  relatedTools: [EApplicationId.SPAWN_PACKER, EApplicationId.SPAWN_EDITOR],
};
