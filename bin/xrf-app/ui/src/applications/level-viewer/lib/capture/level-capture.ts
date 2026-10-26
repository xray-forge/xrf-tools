import { ETextureEncodingFormat } from "@xrf/ipc/types/xrf-texture";

import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

/** What the level viewer's capture takes. */
export enum ELevelCaptureMode {
  /** The level seen straight down, as `demo_record`'s F11. */
  MAP = "map",
  /** The six faces around the camera, as `demo_record`'s Backspace. */
  CUBEMAP = "cubemap",
}

/** The settling frames a tile or face is offered, fewest first. */
export const LEVEL_CAPTURE_SETTLE_FRAMES: ReadonlyArray<number> = [8, 16, 32, 64];

/** Pixels a side every GPU draws a texture at, wgpu's default limit: what a tile or face drawn times over must fit in. */
export const LEVEL_CAPTURE_DRAWN_MOST: number = 8192;

/** The layouts a map or a cubemap is exported to a DDS in, the exports' default first. */
export const LEVEL_CAPTURE_DDS_FORMATS: ReadonlyArray<IChoiceFormRowOption<ETextureEncodingFormat>> = [
  { value: ETextureEncodingFormat.BC7, label: "BC7" },
  { value: ETextureEncodingFormat.BC3, label: "BC3" },
  { value: ETextureEncodingFormat.BC1, label: "BC1" },
  { value: ETextureEncodingFormat.RGBA8, label: "RGBA8" },
];
