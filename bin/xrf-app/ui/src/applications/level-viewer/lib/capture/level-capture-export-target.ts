import { open as openDialog, save } from "@tauri-apps/plugin-dialog";
import { EXrayExtension } from "@xrf/ipc/types/xrf-extension";
import { Nullable } from "@xrf/types";

import { ELevelCaptureMode } from "@/applications/level-viewer/lib/capture/level-capture";
import {
  ILevelCubeExportSettings,
  isLevelCubeExportSingle,
} from "@/applications/level-viewer/lib/cube/level-cube-export";
import {
  ILevelMapExportSettings,
  toLevelMapExportExtension,
} from "@/applications/level-viewer/lib/map/level-map-export";

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

    return save({
      defaultPath: `map_${level}.${extension}`,
      filters: [{ extensions: [extension], name: `${extension.toUpperCase()} image` }],
      title: "Export level map",
    });
  }

  if (isLevelCubeExportSingle(settings.cube.kind)) {
    return save({
      defaultPath: `cube_${level}.${EXrayExtension.DDS}`,
      filters: [{ extensions: [EXrayExtension.DDS], name: "DDS cube texture" }],
      title: "Export cubemap",
    });
  }

  const folder: Nullable<string | Array<string>> = await openDialog({
    directory: true,
    title: "Export the cubemap's faces into a folder",
  });

  return typeof folder === "string" ? folder : null;
}
