import { TextureDescription, TextureDescriptorForm } from "@/core/bindings/types/xrf-app";
import { Nullable } from "@/lib/types/general";

/**
 * The form a texture with no descriptor starts from.
 *
 * What the editor shows before anybody touches it, and nothing more. A save of an authored descriptor applies the form
 * onto `ThmFile::new_texture` in the backend, so what actually reaches disk is the SDK's own defaults; these values
 * only decide what a person sees first. Width and height are zero here because they belong to the DDS header, which
 * `toEditableForm` fills in.
 */
export const EMPTY_TEXTURE_DESCRIPTOR_FORM: TextureDescriptorForm = {
  bumpMode: 0,
  bumpName: "",
  borderColor: 0,
  detailName: "",
  detailScale: 1,
  extNormalMapName: "",
  fadeAmount: 0,
  fadeColor: 0,
  fadeDelay: 0,
  flags: 0,
  format: 0,
  height: 0,
  material: 0,
  materialWeight: 0,
  mipFilter: 0,
  textureType: 0,
  virtualHeight: 0.05,
  width: 0,
};

/**
 * The form to edit for a described texture: the descriptor it has, or a new one shaped by the file beside it.
 *
 * @param description - What the backend answered for the selected texture, or null when none is selected.
 * @returns The form to bind, or null when nothing is selected.
 */
export function toEditableForm(description: Nullable<TextureDescription>): Nullable<TextureDescriptorForm> {
  if (!description) {
    return null;
  }

  const shape = description.base?.shape ?? null;
  const form: TextureDescriptorForm = description.form ?? EMPTY_TEXTURE_DESCRIPTOR_FORM;

  return shape ? { ...form, height: shape.height, width: shape.width } : form;
}

/**
 * Whether two forms describe the same descriptor, field for field.
 *
 * Compared by value rather than by identity, so a field edited back to what it was leaves the node clean. A person who
 * types over a name and types it again has changed nothing, and a dirty flag that said otherwise would prompt them to
 * save a file that is already what they want.
 *
 * @param left - One form.
 * @param right - The other.
 * @returns Whether every field matches.
 */
export function isSameDescriptorForm(
  left: Nullable<TextureDescriptorForm>,
  right: Nullable<TextureDescriptorForm>
): boolean {
  if (left === null || right === null) {
    return left === right;
  }

  return (Object.keys(left) as Array<keyof TextureDescriptorForm>).every((key) => left[key] === right[key]);
}
