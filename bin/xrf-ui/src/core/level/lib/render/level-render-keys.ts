/** The keys a level is put to the renderer under. */
export const LEVEL_RENDER_KEYS = {
  axes: "axes",
  extent: "extent",
  extentBox: "extent-box",
  grid: "grid",
  /** One run of a sector's impostors drawn with one surface. */
  impostorGroup: (sector: number, group: number): string => `sector:${sector}:impostors:${group}`,
  /** The quad every impostor draws over, its corners numbered in its coordinate. */
  impostorQuad: "impostor-quad",
  /** A sector's impostor set, which its trees and its impostor draws both name. */
  impostors: (sector: number): string => `sector:${sector}:impostors`,
  /** One mesh a sector stands in many places. */
  instance: (sector: number, index: number): string => `sector:${sector}:instance:${index}`,
  /** Everything a sector bakes in place, one geometry with a group per surface. */
  sector: (sector: number): string => `sector:${sector}`,
  sun: "sun",
  /** One shader table entry, which every sector drawing it shares. */
  surface: (shaderId: number): string => `surface:${shaderId}`,
} as const;
