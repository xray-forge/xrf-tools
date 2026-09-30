import { saturate } from "@xrf/math";
import { Nullable } from "@xrf/types";
import { renderGroup, texture3D, uniform } from "three/tsl";
import {
  Data3DTexture,
  LinearFilter,
  RepeatWrapping,
  RGBAFormat,
  Texture3DNode,
  TextureNode,
  UniformNode,
  UnsignedByteType,
} from "three/webgpu";

import { IRendererRainfall } from "#/contract/renderer-rainfall";
import { getNeutralDetailTexture } from "#/texture/placeholder-textures";
import { markRendererTextureNew } from "#/texture/renderer-texture-version";
import { SlotTextureNode } from "#/texture/slot-texture-node";

/**
 * What `rain_patch_normal` reads: how hard it rains, the time the ripples run by, and the two textures it wets surfaces
 * with.
 */
export class WetUniforms {
  /** `RainDensity.x`. */
  public readonly density: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** `timers.x`: seconds the renderer has been running. */
  public readonly time: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** `s_water`: nothing rippling until the volume is decoded. */
  public readonly splash: Texture3DNode = texture3D(createNeutralVolume());
  /** `s_waterFall`: flat until its file is up. */
  public readonly flow: TextureNode = new SlotTextureNode(getNeutralDetailTexture());

  /**
   * @param rain - How hard it rains now, or null for no rain.
   */
  public take(rain: Nullable<IRendererRainfall>): void {
    this.density.value = rain ? saturate(rain.density) : 0;
  }

  /**
   * @param time - Seconds the renderer has been running.
   */
  public update(time: number): void {
    this.time.value = time;
  }
}

/** A texel of a half in every channel, which ripples nothing. */
function createNeutralVolume(): Data3DTexture {
  const volume: Data3DTexture = new Data3DTexture(new Uint8Array([128, 128, 128, 128]), 1, 1, 1);

  prepareVolume(volume);

  return volume;
}

/**
 * @param volume - A volume of four bytes a texel, sampled as `smp_base` samples: repeating on every axis, linear.
 */
export function prepareVolume(volume: Data3DTexture): void {
  volume.format = RGBAFormat;
  volume.type = UnsignedByteType;
  volume.wrapS = volume.wrapT = volume.wrapR = RepeatWrapping;
  volume.minFilter = volume.magFilter = LinearFilter;
  volume.generateMipmaps = false;
  markRendererTextureNew(volume);
}
