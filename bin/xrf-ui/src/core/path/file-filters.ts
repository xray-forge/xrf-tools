import { DialogFilter } from "@tauri-apps/plugin-dialog";

import { EXrayExtension } from "@/core/ipc/types/xrf-extension";

/** A packed `all.spawn`, or any other spawn file. */
export const SPAWN_FILE_FILTERS: Array<DialogFilter> = [{ name: "spawn", extensions: [EXrayExtension.SPAWN] }];

/** An OGF visual. */
export const OGF_FILE_FILTERS: Array<DialogFilter> = [{ name: "Ogf visual", extensions: [EXrayExtension.OGF] }];

/** Either half of a texture: its DDS, or the `.thm` descriptor beside it. */
export const TEXTURE_FILE_FILTERS: Array<DialogFilter> = [
  { name: "Texture or descriptor", extensions: [EXrayExtension.DDS, EXrayExtension.THM] },
];

/** A DDS texture on its own. */
export const DDS_FILE_FILTERS: Array<DialogFilter> = [{ name: "dds", extensions: [EXrayExtension.DDS] }];

/** An LTX configuration. */
export const LTX_FILE_FILTERS: Array<DialogFilter> = [{ name: "ltx", extensions: [EXrayExtension.LTX] }];

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
