import { MeshBasicNodeMaterial, TextureNode } from "three/webgpu";

/** A material the weather draws with, rain or a bolt, and the sampler its texture is bound to. */
export interface IWeatherSurface {
  material: MeshBasicNodeMaterial;
  /** Samples nothing until its texture is bound. */
  texture: TextureNode;
}
