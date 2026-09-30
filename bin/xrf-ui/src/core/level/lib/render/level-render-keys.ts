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
  /** A visual the level's spawned objects stand as: its submeshes joined in one geometry, a group each. */
  spawnGeometry: (visual: number): string => `spawn:${visual}`,
  /** The objects of one category standing as a visual. */
  spawnObject: (visual: number, category: string): string => `spawn:${visual}:${category}`,
  /** One submesh's surface of a visual, which every category's objects of it draw with. */
  spawnSurface: (visual: number, submesh: number): string => `spawn:${visual}:surface:${submesh}`,
  sun: "sun",
  /** One shader table entry, which every sector drawing it shares. */
  surface: (shaderId: number): string => `surface:${shaderId}`,
} as const;
