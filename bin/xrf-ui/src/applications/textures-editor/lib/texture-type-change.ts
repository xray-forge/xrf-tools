import { TextureDescriptorForm, TextureTypeChange } from "@xrf/ipc/types/xrf-app-textures";

/** The fields a change of texture type writes. */
export type TextureTypePatch = Pick<TextureDescriptorForm, "textureType" | "flags" | "format" | "mipFilter">;

/**
 * The fields to write when a descriptor is given a texture type, as the SDK's editor writes them.
 *
 * The SDK's editor does more than store the type: a normal map gets mip maps and a Kaiser filter, a terrain loses its
 * mip maps and is lit implicitly, and both drop the flags only the converter's colour path reads. A type the backend
 * names no change for, a newer one or a number nobody named, is stored alone.
 *
 * @param form - The descriptor as the editor has it.
 * @param textureType - The type to give it.
 * @param changes - What each type changes, from the backend's vocabulary.
 * @returns The patch to apply, which always carries the type.
 */
export function withTextureType(
  form: TextureTypePatch,
  textureType: number,
  changes: ReadonlyArray<TextureTypeChange>
): TextureTypePatch {
  const change: TextureTypeChange | undefined = changes.find(
    (candidate: TextureTypeChange) => candidate.textureType === textureType
  );

  if (!change) {
    return { ...form, textureType };
  }

  return {
    textureType,
    // Unsigned, because the backend reads the word as a u32 and a set top bit would otherwise cross as negative.
    flags: ((form.flags & ~change.clearedFlags) | change.setFlags) >>> 0,
    format: change.format ?? form.format,
    mipFilter: change.mipFilter ?? form.mipFilter,
  };
}
