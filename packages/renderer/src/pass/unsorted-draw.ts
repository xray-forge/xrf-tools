import { WebGPURenderer } from "three/webgpu";

/**
 * Draws with three's sorting off, for a pass whose order changes nothing drawn, and leaves the sorting as it found it.
 *
 * @param renderer - The renderer drawing.
 * @param draw - What is drawn unsorted.
 */
export function drawUnsorted(renderer: WebGPURenderer, draw: () => void): void {
  const isSorted: boolean = renderer.sortObjects;

  renderer.sortObjects = false;

  try {
    draw();
  } finally {
    renderer.sortObjects = isSorted;
  }
}
