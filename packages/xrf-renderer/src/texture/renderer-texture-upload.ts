import { Nullable } from "@xrf/types";
import { Texture } from "three/webgpu";

import { IDdsRefusal } from "#/dds/dds-refusal";

/** What a dds upload came to: exactly one of the two is present. */
export interface IRendererTextureUpload {
  texture: Nullable<Texture>;
  refusal: Nullable<IDdsRefusal>;
}
