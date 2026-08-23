import { BufferAttribute, BufferGeometry, DataTexture, NearestFilter, RepeatWrapping, RGBAFormat } from "three";

import { IVisualPreviewSceneConfig } from "@/core/visuals/components/scene/scene-config";
import { getVisualSubmeshLevel, IVisualSubmeshLevel, IVisualSubmeshViews } from "@/core/visuals/lib/visual-views";

/**
 * A small procedural checkerboard, built as raw pixels rather than through a canvas so it needs no dom.
 *
 * @param config - Scene configuration supplying the board's size and repeat.
 * @returns A repeating, nearest-filtered checkerboard texture.
 */
export function createCheckerTexture(config: IVisualPreviewSceneConfig): DataTexture {
  const { checkerSize, checkerRepeat } = config;
  const data: Uint8Array = new Uint8Array(checkerSize * checkerSize * 4);

  for (let y = 0; y < checkerSize; y++) {
    for (let x = 0; x < checkerSize; x++) {
      const offset: number = (y * checkerSize + x) * 4;
      const value: number = (x + y) % 2 === 0 ? 0xff : 0x40;

      data[offset] = value;
      data[offset + 1] = value;
      data[offset + 2] = value;
      data[offset + 3] = 0xff;
    }
  }

  const texture: DataTexture = new DataTexture(data, checkerSize, checkerSize, RGBAFormat);

  texture.wrapS = RepeatWrapping;
  texture.wrapT = RepeatWrapping;
  texture.repeat.set(checkerRepeat, checkerRepeat);
  texture.magFilter = NearestFilter;
  texture.minFilter = NearestFilter;
  texture.needsUpdate = true;

  return texture;
}

/**
 * Build one submesh's geometry, drawing the range of the chosen detail level.
 *
 * `setDrawRange` rather than a trimmed index buffer, because every detail level stays in the buffer: switching level
 * is a different range over the same upload, which is why the picker costs no re-read and no re-upload.
 *
 * @param submesh - Views over the shared geometry buffer, and the ranges to draw from them.
 * @param detail - How far down this submesh's collapse chain to draw: 0 is full detail, 1 is its coarsest.
 * @returns Geometry ready to upload.
 */
export function createSubmeshGeometry(submesh: IVisualSubmeshViews, detail: number): BufferGeometry {
  const geometry: BufferGeometry = new BufferGeometry();

  geometry.setAttribute("position", new BufferAttribute(submesh.positions, 3));
  geometry.setAttribute("normal", new BufferAttribute(submesh.normals, 3));
  geometry.setAttribute("uv", new BufferAttribute(submesh.uvs, 2));
  geometry.setIndex(new BufferAttribute(submesh.indices, 1));

  const level: IVisualSubmeshLevel = getVisualSubmeshLevel(submesh, detail);

  geometry.setDrawRange(level.start, level.count);
  geometry.computeBoundingSphere();

  return geometry;
}
