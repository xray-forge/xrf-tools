/**
 * @param hex - A hex colour as a page writes one.
 * @returns Its bytes as raw values, which is what the renderer's overlays draw.
 */
export function toRawColor(hex: number): [number, number, number] {
  return [((hex >> 16) & 0xff) / 255, ((hex >> 8) & 0xff) / 255, (hex & 0xff) / 255];
}
