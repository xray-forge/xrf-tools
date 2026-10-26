import { RenderRect } from "@xrf/ipc/types/xrf-renderer";

import { ICssBox } from "@/lib/dom/css-box";

/**
 * @param box - A box of the page in CSS pixels.
 * @param scale - Device pixels a CSS pixel.
 * @returns The same box in device pixels of the window's client area, each edge rounded as the page lays it out, so
 *   two boxes sharing an edge share it on the device too.
 */
export function toNativeRect(box: ICssBox, scale: number): RenderRect {
  const left: number = Math.round(box.left * scale);
  const top: number = Math.round(box.top * scale);

  return {
    height: Math.max(0, Math.round(box.bottom * scale) - top),
    width: Math.max(0, Math.round(box.right * scale) - left),
    x: left,
    y: top,
  };
}
