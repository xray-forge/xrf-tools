import { Nullable } from "@xrf/types";
import { Texture } from "three/webgpu";

import { IRendererTextureSize } from "#/contract/scene/renderer-texture-size";
import { IDdsRefusal } from "#/dds/dds-refusal";

/** What a dds upload came to: a texture and the file's size, or the refusal. */
export interface IRendererTextureUpload {
  texture: Nullable<Texture>;
  refusal: Nullable<IDdsRefusal>;
  size: Nullable<IRendererTextureSize>;
}
