import { EXrayExtension } from "@xrf/ipc/types/xrf-extension";
import { Nullable } from "@xrf/types";

import { askToOpenDirectory, askToSave } from "@/core/path/native-dialog";
import { ELevelCaptureMode } from "@/tools/level-viewer/lib/capture/level-capture";
import { ILevelCubeExportSettings, isLevelCubeExportSingle } from "@/tools/level-viewer/lib/cube/level-cube-export";
import { ILevelMapExportSettings, toLevelMapExportExtension } from "@/tools/level-viewer/lib/map/level-map-export";

/** How the next export of a map and of a cubemap is written, which decides what the dialog asks for. */
export interface ILevelCaptureExportSettings {
  map: ILevelMapExportSettings;
  cube: ILevelCubeExportSettings;
}

/**
 * Asks where a capture's export goes: a file for a map or a single cubemap texture, a folder for six faces.
 *
 * @param mode - What was captured.
 * @param level - The open level's name, which the file is named after.
 * @param settings - How the export is written.
 * @returns Where it goes, as the save or folder dialog answers, or null where it was dismissed.
 */
export async function askLevelCaptureExportTarget(
  mode: ELevelCaptureMode,
  level: string,
  settings: ILevelCaptureExportSettings
): Promise<Nullable<string>> {
  if (mode === ELevelCaptureMode.MAP) {
    const extension: string = toLevelMapExportExtension(settings.map.kind);

    return askToSave({
      filters: [{ extensions: [extension], name: `${extension.toUpperCase()} image` }],
      start: `map_${level}.${extension}`,
      title: "Export level map",
    });
  }

  if (isLevelCubeExportSingle(settings.cube.kind)) {
    return askToSave({
      filters: [{ extensions: [EXrayExtension.DDS], name: "DDS cube texture" }],
      start: `cube_${level}.${EXrayExtension.DDS}`,
      title: "Export cubemap",
    });
  }

  return askToOpenDirectory({ title: "Export the cubemap's faces into a folder" });
}
