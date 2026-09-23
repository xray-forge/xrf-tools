/** The keys a level is put to the renderer under. */
export const LEVEL_RENDER_KEYS = {
  axes: "axes",
  extent: "extent",
  extentBox: "extent-box",
  grid: "grid",
  /** One mesh a sector stands in many places. */
  instance: (sector: number, index: number): string => `sector:${sector}:instance:${index}`,
  /** Everything a sector bakes in place, one geometry with a group per surface. */
  sector: (sector: number): string => `sector:${sector}`,
  sun: "sun",
  /** One shader table entry, which every sector drawing it shares. */
  surface: (shaderId: number): string => `surface:${shaderId}`,
} as const;
