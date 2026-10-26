import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const SPAWN_EDITOR_HELP: IApplicationHelp = {
  summary:
    "Inspect a packed `.spawn` file by chunk: header, ALife objects, artefact nodes, patrols, and game graph. " +
    "Save the open spawn to a chosen file or export its chunks for work outside the application.",
  workflow: [
    "Choose a `.spawn` file and select `Open`. Opening reads the file without writing anything.",
    "Choose a chunk in `Chunks`, then use its tabs and tables to inspect the data. Select a row for its details.",
    "Use `Save spawn file` to choose a packed output file, or `Export spawn file` to choose a directory of chunks.",
  ],
  nuances: [
    "The status bar reports the file version and the object and level counts from its header.",
    "Saving asks for an output path each time. Exporting writes one file per chunk and asks for confirmation " +
      "before replacing an unpacked spawn in the destination.",
  ],
  limitations: [
    "The current chunk tables are for inspection; they do not offer inline edits or object creation.",
    "Saving and exporting write files to disk. Export replaces existing chunk files in the chosen directory.",
  ],
  relatedTools: [EApplicationId.SPAWN_UNPACKER, EApplicationId.SPAWN_PACKER],
};
