import { HalfFloatType, LinearFilter, NearestFilter, RenderTarget, RGBAFormat, Texture } from "three/webgpu";

import { IColourAttachment } from "#/pass/colour-attachment";

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

  colours.forEach(
    ({ name, format = RGBAFormat, type = HalfFloatType, isFiltered = true }: IColourAttachment, index: number) => {
      const texture: Texture = target.textures[index];

      texture.name = name;
      texture.format = format;
      texture.type = type;
      texture.minFilter = isFiltered ? LinearFilter : NearestFilter;
      texture.magFilter = isFiltered ? LinearFilter : NearestFilter;
      texture.generateMipmaps = false;
    }
  );

  return target;
}
