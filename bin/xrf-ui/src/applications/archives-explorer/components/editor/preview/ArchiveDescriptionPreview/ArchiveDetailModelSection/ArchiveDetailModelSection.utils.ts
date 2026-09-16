import { ArchiveBounds, ArchiveDetailModel } from "@/core/ipc/types/xrf-app";
import { formatNumber } from "@/lib/format/number";
import { Nullable } from "@/lib/types/general";

/**
 * How much space a detail model occupies as authored.
 *
 * @param bounds - Extents the mesh spans, or null for a model carrying none.
 * @returns The three extents in engine units, which are metres, or null where there is no mesh to measure.
 */
export function describeModelBounds(bounds: Nullable<ArchiveBounds>): Nullable<string> {
  if (!bounds) {
    return null;
  }

  const width: string = formatNumber(bounds.width, 2);
  const height: string = formatNumber(bounds.height, 2);
  const depth: string = formatNumber(bounds.depth, 2);

  return `${width} × ${height} × ${depth} m`;
}

/**
 * The range a slot may scale the model by when it plants one.
 *
 * @param model - Model to describe.
 * @returns The range, or the single value where the two ends are the same number.
 */
export function describeModelScale(model: ArchiveDetailModel): string {
  const minimum: string = formatNumber(model.minScale, 2);
  const maximum: string = formatNumber(model.maxScale, 2);

  return minimum === maximum ? minimum : `${minimum}–${maximum}`;
}

/**
 * What the mesh of a detail model is made of.
 *
 * @param model - Model to describe.
 * @returns A phrase for its size.
 */
export function describeModelMesh(model: ArchiveDetailModel): string {
  const triangles: string = `${model.triangles} ${model.triangles === 1 ? "triangle" : "triangles"}`;

  return `${triangles} over ${model.vertices} ${model.vertices === 1 ? "vertex" : "vertices"}`;
}

/**
 * What qualifies a model beneath its name: how it is scaled, whether it sways, and any flag bit nothing claims.
 *
 * @param model - Model to describe.
 * @returns The qualifying phrases, already in reading order.
 */
export function describeModelDetail(model: ArchiveDetailModel): string {
  const detail: Array<string> = [describeModelMesh(model), `scale ${describeModelScale(model)}`];

  // Named only where it is off: the renderer sways a detail object unless the file says not to, so the flag being
  // clear is the ordinary case and saying so on every row would be noise.
  if (!model.isWaving) {
    detail.push("does not sway");
  }

  if (model.unnamedFlags) {
    detail.push(`unnamed bits 0x${(model.unnamedFlags >>> 0).toString(16).toUpperCase()}`);
  }

  return detail.join(" · ");
}
