import {
  AnyPixelFormat,
  HalfFloatType,
  LinearFilter,
  NearestFilter,
  RenderTarget,
  RGBAFormat,
  Texture,
  TextureDataType,
} from "three/webgpu";

/** One colour a target writes. */
export interface IColourAttachment {
  /** What the device labels it. */
  name: string;
  /** Four channels unless said. */
  format?: AnyPixelFormat;
  /** Half floats unless said. */
  type?: TextureDataType;
  /** Whether it is sampled between texels; read texel by texel otherwise. */
  isFiltered?: boolean;
}

/**
 * @param colours - What the target writes, in its attachments' order.
 * @param isDepthed - Whether it has a depth attachment.
 * @returns The target, each colour set up as asked.
 */
export function createColourTarget(
  colours: ReadonlyArray<IColourAttachment>,
  isDepthed: boolean = false
): RenderTarget {
  const target: RenderTarget = new RenderTarget(1, 1, { count: colours.length, depthBuffer: isDepthed });

  colours.forEach(({ name, format = RGBAFormat, type = HalfFloatType, isFiltered = true }, index: number) => {
    const texture: Texture = target.textures[index];

    texture.name = name;
    texture.format = format;
    texture.type = type;
    texture.minFilter = isFiltered ? LinearFilter : NearestFilter;
    texture.magFilter = isFiltered ? LinearFilter : NearestFilter;
    texture.generateMipmaps = false;
  });

  return target;
}
