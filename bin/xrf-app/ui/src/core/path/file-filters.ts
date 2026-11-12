import { DialogFilter } from "@tauri-apps/plugin-dialog";
import { EXrayExtension } from "@xrf/ipc/types/xrf-extension";

/** A packed `all.spawn`, or any other spawn file. */
export const SPAWN_FILE_FILTERS: Array<DialogFilter> = [{ name: "spawn", extensions: [EXrayExtension.SPAWN] }];

/** An OGF visual. */
export const OGF_FILE_FILTERS: Array<DialogFilter> = [{ name: "Ogf visual", extensions: [EXrayExtension.OGF] }];

/** A model to view: a compiled OGF, or the SDK object it is exported from. */
export const VISUAL_SOURCE_FILE_FILTERS: Array<DialogFilter> = [
  { name: "Ogf visual or SDK object", extensions: [EXrayExtension.OGF, EXrayExtension.OBJECT] },
];

/** Either half of a texture: its DDS, or the `.thm` descriptor beside it. */
export const TEXTURE_FILE_FILTERS: Array<DialogFilter> = [
  { name: "Texture or descriptor", extensions: [EXrayExtension.DDS, EXrayExtension.THM] },
];

/** A DDS texture on its own. */
export const DDS_FILE_FILTERS: Array<DialogFilter> = [{ name: "dds", extensions: [EXrayExtension.DDS] }];

/** An LTX configuration. */
export const LTX_FILE_FILTERS: Array<DialogFilter> = [{ name: "ltx", extensions: [EXrayExtension.LTX] }];

/** An SDK Level Editor scene's main file, which no game format shares an extension with. */
export const LEVEL_SCENE_FILE_FILTERS: Array<DialogFilter> = [{ name: "Level Editor scene", extensions: ["level"] }];

/** A library the SDK keeps as `.xr`, such as the game materials' `gamemtl.xr`. */
export const XR_LIBRARY_FILE_FILTERS: Array<DialogFilter> = [{ name: "xr library", extensions: [EXrayExtension.XR] }];

/**
 * One save dialog entry per format, so the extension a person picks is what decides the writer.
 *
 * @param formats - The formats, in the order the dialog offers them.
 * @param toName - What an entry is called, from its format.
 * @returns The filters.
 */
export function toFormatFilters(
  formats: ReadonlyArray<EXrayExtension>,
  toName: (format: EXrayExtension) => string
): Array<DialogFilter> {
  return formats.map((format: EXrayExtension) => ({ name: toName(format), extensions: [format] }));
}
