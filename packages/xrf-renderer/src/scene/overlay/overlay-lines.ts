import { BufferGeometry, LineBasicNodeMaterial, LineSegments } from "three/webgpu";

import { OVERLAY_RENDER_ORDER } from "#/scene/overlay/overlay-drawing";

/**
 * @param geometry - The segments.
 * @param isDepthTested - Whether what stands in front hides them.
 * @param hasColors - Whether the geometry colours each vertex.
 * @returns Segments drawn over the frame, unlit.
 */
export function createOverlayLines(geometry: BufferGeometry, isDepthTested: boolean, hasColors: boolean): LineSegments {
  const material: LineBasicNodeMaterial = new LineBasicNodeMaterial({ vertexColors: hasColors });
  const lines: LineSegments = new LineSegments(geometry, material);

  material.depthTest = isDepthTested;
  material.depthWrite = false;
  lines.renderOrder = OVERLAY_RENDER_ORDER;

  return lines;
}
