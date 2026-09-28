import { Texture } from "three/webgpu";

/** A level or image with its bytes let go: its size alone. */
interface ILevelSize {
  width: number;
  height: number;
}

/**
 * Lets go of a texture's bytes once it is on the GPU: every level keeps its size, and its data is nothing. Its source
 * is marked not ready, so a texture three makes again after it was disposed (a binding built over it before, which
 * three uploads anew) allocates and reads nothing, rather than throwing on bytes that are gone.
 *
 * @param texture - A texture three uploaded and never has to send again: no `needsUpdate` from here on.
 */
export function releaseTextureData(texture: Texture): void {
  const image: unknown = texture.image;

  texture.mipmaps = texture.mipmaps.map((level: unknown) => toSize(level)) as unknown as Texture["mipmaps"];

  if (typeof ImageBitmap !== "undefined" && image instanceof ImageBitmap) {
    image.close();
  }

  texture.image = toSize(image);
  texture.source.dataReady = false;
}

/**
 * @param texture - A texture.
 * @returns Whether it still holds its bytes: false once `releaseTextureData` let them go.
 */
export function hasTextureData(texture: Texture): boolean {
  return texture.source.dataReady !== false;
}

/**
 * @param texture - A texture.
 * @returns The byte arrays it holds on the CPU: its levels' and its image's.
 */
export function listTextureData(texture: Texture): Array<ArrayBufferView> {
  return [texture.image, ...texture.mipmaps]
    .map((level: unknown) => (level as { data?: unknown } | null)?.data)
    .filter((data: unknown): data is ArrayBufferView => ArrayBuffer.isView(data));
}

/** A level's or image's size, whatever it held: a picture, a level's blocks, or texels. */
function toSize(level: unknown): ILevelSize & { data: null } {
  const { width, height } = (level ?? {}) as Partial<ILevelSize>;

  return { data: null, height: height ?? 1, width: width ?? 1 };
}
