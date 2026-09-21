import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";

/**
 * One row of a level's shader table, as a reader asking how it is drawn needs it.
 */
export interface ILevelSurfaceSummary {
  /** Position in the table, which is the shader id every surface of the level refers to it by. */
  shaderId: number;
  /** The shader the entry names, or a stand-in for an entry that names none. */
  shader: string;
  descriptor: XraySurfaceDescriptor;
}

/** What an entry naming no shader is called, since the table keeps its place either way. */
export const UNNAMED_LEVEL_SURFACE: string = "(no shader)";

/**
 * Summarises a level's shader table, one row per entry, in the table's own order.
 *
 * @param surfaces - The table as the backend resolved it.
 * @returns One row per entry.
 */
export function listLevelSurfaces(surfaces: ReadonlyArray<XraySurfaceDescriptor>): Array<ILevelSurfaceSummary> {
  return surfaces.map((descriptor: XraySurfaceDescriptor, shaderId: number) => ({
    descriptor,
    shader: descriptor.shader ?? UNNAMED_LEVEL_SURFACE,
    shaderId,
  }));
}

/**
 * The same table with the entries nothing names dropped.
 *
 * @param summaries - Every row.
 * @returns The rows naming a shader.
 */
export function listNamedLevelSurfaces(summaries: ReadonlyArray<ILevelSurfaceSummary>): Array<ILevelSurfaceSummary> {
  return summaries.filter((summary: ILevelSurfaceSummary) => summary.descriptor.shader !== null);
}
