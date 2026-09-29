import { Texture } from "three/webgpu";

/** The highest version given out, which the next texture made passes. */
let latest: number = 0;

/**
 * Marks a texture made for upload with a version no other texture has, in place of `needsUpdate`. Three rebuilds a
 * bind group for a sampler pointed at another texture only where the two versions differ (`Bindings._update`, its
 * generation), so two textures both at version one would keep drawing the first.
 *
 * @param texture - A texture just made, not yet uploaded.
 */
export function markRendererTextureNew(texture: Texture): void {
  latest = Math.max(latest, texture.version) + 1;
  texture.version = latest;
}
