import { clamp, float, floor, select, sqrt, uint } from "three/tsl";
import { Node, StorageBufferNode } from "three/webgpu";

import { toStaticScreenArea } from "#/scene/static/static-screen-area.tsl";
import { LodUniforms } from "#/uniforms/lod-uniforms";
import { STATIC_LOD_IMPOSTOR_ROW, STATIC_NO_BAND, STATIC_NO_LOD } from "#/uniforms/static-draw-buffers";
import { EStaticLodState } from "#/uniforms/static-lod-state";

/**
 * Whether a row's clump draws what the row is: a tree while its trees are near enough, the impostor's own draw while it
 * is far enough; a row no impostor stands in for always.
 *
 * @param row - The row's impostor word, `STATIC_NO_LOD` for none.
 * @param terms - What the LOD cull decided, an impostor each.
 */
export function toLodDrawn(row: Node<"uint">, terms: StorageBufferNode<"uvec4">): Node<"bool"> {
  const lod: Node<"uint"> = row.bitAnd(STATIC_LOD_IMPOSTOR_ROW - 1);
  const wanted: Node<"uint"> = select(
    row.bitAnd(STATIC_LOD_IMPOSTOR_ROW).notEqual(0),
    uint(EStaticLodState.IMPOSTOR),
    uint(EStaticLodState.TREES)
  );

  return row.equal(STATIC_NO_LOD).or((terms.element(lod) as unknown as Node<"uvec4">).w.bitAnd(wanted).notEqual(0));
}

/**
 * @param row - A row's impostor word.
 * @returns Whether the row is an impostor's own draw, which a shadow view never casts (`add_leafs_static`).
 */
export function toImpostorRow(row: Node<"uint">): Node<"bool"> {
  return row.notEqual(STATIC_NO_LOD).and(row.bitAnd(STATIC_LOD_IMPOSTOR_ROW).notEqual(0));
}

/**
 * Whether a row's band is the one its place draws at (`calcLOD`, `FTreeVisual_PM::Render`): the place's screen area
 * against the progressive thresholds gives its detail, the detail a window of the engine's table, and the window the
 * band it falls in. A row of a draw of one detail always does.
 *
 * @param word - The row's band word, `STATIC_NO_BAND` for none.
 * @param sphere - The place's sphere.
 * @param lod - The thresholds and the camera they are measured from.
 */
export function toBandDrawn(word: Node<"uint">, sphere: Node<"vec4">, lod: LodUniforms): Node<"bool"> {
  const band: Node<"uint"> = word.bitAnd(255);
  const bands: Node<"uint"> = word.shiftRight(8).bitAnd(255);
  const windows: Node<"uint"> = word.shiftRight(16);
  const offset: Node<"vec3"> = sphere.xyz.sub(lod.camera);
  const ssa: Node<"float"> = toStaticScreenArea(sphere.w, offset);
  const detail: Node<"float"> = sqrt(clamp(ssa.sub(lod.glodEnd).div(lod.glodStart.sub(lod.glodEnd)), 0, 1));
  const window: Node<"uint"> = floor(
    float(1)
      .sub(detail)
      .mul(float(windows.sub(1)))
      .add(0.5)
  ).toUint();

  // The window is at most `windows - 1`, so the band it falls in is always one of the draw's.
  return word.equal(STATIC_NO_BAND).or(window.mul(bands).div(windows).equal(band));
}

/**
 * Whether a row's band is the finest, which a light's face casts whatever the camera: the face is kept while nothing it
 * casts from changes, so a band picked from where the camera stood would stay as the camera comes near.
 *
 * @param word - The row's band word, `STATIC_NO_BAND` for none.
 */
export function toFinestBand(word: Node<"uint">): Node<"bool"> {
  return word.equal(STATIC_NO_BAND).or(word.bitAnd(255).equal(0));
}
