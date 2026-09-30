import { Nullable } from "@xrf/types";

import { TextureDescription, TextureDescriptorForm } from "@/core/ipc/types/xrf-app";

/**
 * The form to edit for a described texture, shaped by the file beside it.
 *
 * @param description - What the backend answered for the selected texture, or null when none is selected.
 * @returns The form to bind, or null when nothing is selected.
 */
export function toEditableForm(description: Nullable<TextureDescription>): Nullable<TextureDescriptorForm> {
  if (!description) {
    return null;
  }

  const shape = description.base?.shape ?? null;
  // The backend's own for a texture with no descriptor: `ThmFile::new_texture`, the defaults the SDK authors with.
  const form: TextureDescriptorForm = description.form;

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
