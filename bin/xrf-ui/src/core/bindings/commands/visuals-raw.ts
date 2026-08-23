// Auto-generated rust bindings. Do not edit it manually.

import { VisualSource } from "@/core/bindings/types/xrf-app";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { invokeRaw } from "@/core/ipc/raw";

/** Commands answering with raw bytes, which Specta cannot type. */
export const visualsRawCommands = {
  readGeometry: (source: VisualSource, roots: XrayRoots): Promise<ArrayBuffer> =>
    invokeRaw("plugin:visuals|read_geometry", { source, roots }),
  readMotion: (name: string): Promise<ArrayBuffer> => invokeRaw("plugin:visuals|read_motion", { name }),
};
