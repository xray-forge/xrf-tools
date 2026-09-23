import { IRendererGeometry } from "#/contract/scene/renderer-geometry";

/** The last index a 16 bit index buffer can hold. */
const SHORT_INDEX_LIMIT: number = 65_535;

/**
 * A uv sphere centred on the origin, laid out as three's `SphereGeometry` is, with uvs top first.
 *
 * @param radius - Its radius.
 * @param widthSegments - Segments around it.
 * @param heightSegments - Segments from pole to pole.
 * @returns The geometry, in one group.
 */
export function createRendererSphere(radius: number, widthSegments: number, heightSegments: number): IRendererGeometry {
  const position: Array<number> = [];
  const normal: Array<number> = [];
  const uv: Array<number> = [];
  const index: Array<number> = [];
  const grid: Array<Array<number>> = [];

  for (let row = 0; row <= heightSegments; row += 1) {
    const v: number = row / heightSegments;
    // The poles' uvs shift half a segment, so each pole triangle samples the middle of its own column.
    const offset: number = row === 0 ? 0.5 / widthSegments : row === heightSegments ? -0.5 / widthSegments : 0;
    const vertices: Array<number> = [];

    for (let column = 0; column <= widthSegments; column += 1) {
      const u: number = column / widthSegments;
      const x: number = -Math.cos(u * Math.PI * 2) * Math.sin(v * Math.PI);
      const y: number = Math.cos(v * Math.PI);
      const z: number = Math.sin(u * Math.PI * 2) * Math.sin(v * Math.PI);

      vertices.push(position.length / 3);
      position.push(x * radius, y * radius, z * radius);
      normal.push(x, y, z);
      uv.push(u + offset, v);
    }

    grid.push(vertices);
  }

  for (let row = 0; row < heightSegments; row += 1) {
    for (let column = 0; column < widthSegments; column += 1) {
      const a: number = grid[row][column + 1];
      const b: number = grid[row][column];
      const c: number = grid[row + 1][column];
      const d: number = grid[row + 1][column + 1];

      if (row !== 0) {
        index.push(a, b, d);
      }

      if (row !== heightSegments - 1) {
        index.push(b, c, d);
      }
    }
  }

  return {
    groups: [{ count: index.length, slot: 0, start: 0 }],
    index: position.length / 3 > SHORT_INDEX_LIMIT ? new Uint32Array(index) : new Uint16Array(index),
    normal: new Float32Array(normal),
    position: new Float32Array(position),
    uv: new Float32Array(uv),
  };
}
