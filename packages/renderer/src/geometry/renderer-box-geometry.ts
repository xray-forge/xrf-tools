import { IRendererGeometry, IRendererGeometryGroup } from "#/contract/scene/renderer-geometry";

/** An axis of a face: across it, down it, or out of it. */
type TAxis = 0 | 1 | 2;

/** One face as three's `BoxGeometry` builds it: its axes, their directions, and the extents along them. */
type TBoxFace = [TAxis, TAxis, TAxis, number, number, number, number, number];

/**
 * A box centred on the origin, a group per face in three's order (`+x`, `-x`, `+y`, `-y`, `+z`, `-z`), uvs top first.
 *
 * @param width - Extent along x.
 * @param height - Extent along y.
 * @param depth - Extent along z.
 * @returns The geometry, slot `n` drawing face `n`.
 */
export function createRendererBox(width: number, height: number, depth: number): IRendererGeometry {
  const position: Array<number> = [];
  const normal: Array<number> = [];
  const uv: Array<number> = [];
  const index: Array<number> = [];
  const groups: Array<IRendererGeometryGroup> = [];

  const faces: ReadonlyArray<TBoxFace> = [
    [2, 1, 0, -1, -1, depth, height, width],
    [2, 1, 0, 1, -1, depth, height, -width],
    [0, 2, 1, 1, 1, width, depth, height],
    [0, 2, 1, 1, -1, width, depth, -height],
    [0, 1, 2, 1, -1, width, height, depth],
    [0, 1, 2, -1, -1, width, height, -depth],
  ];

  faces.forEach(([across, down, out, acrossSign, downSign, faceWidth, faceHeight, faceDepth], slot: number) => {
    const first: number = position.length / 3;

    for (let row = 0; row < 2; row += 1) {
      for (let column = 0; column < 2; column += 1) {
        const vertex: [number, number, number] = [0, 0, 0];
        const facing: [number, number, number] = [0, 0, 0];

        vertex[across] = (column - 0.5) * faceWidth * acrossSign;
        vertex[down] = (row - 0.5) * faceHeight * downSign;
        vertex[out] = faceDepth / 2;
        facing[out] = faceDepth > 0 ? 1 : -1;

        position.push(...vertex);
        normal.push(...facing);
        // Top first, as X-Ray stores rows: the file's first row on the face's first row.
        uv.push(column, row);
      }
    }

    // Wound as three winds them, so every face is front facing from outside.
    index.push(first, first + 2, first + 1, first + 2, first + 3, first + 1);
    groups.push({ count: 6, slot, start: slot * 6 });
  });

  return {
    groups,
    index: new Uint16Array(index),
    normal: new Float32Array(normal),
    position: new Float32Array(position),
    uv: new Float32Array(uv),
  };
}
