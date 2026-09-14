import { EquipmentSpriteMetadata } from "@/core/ipc/types/xrf-app";
import { EquipmentSlotOccupant } from "@/core/ipc/types/xrf-texture";
import { IEquipmentLayout } from "@/core/sprite-equipment/lib";
import { Nullable } from "@/lib/types/general";

/** What kind of thing is wrong, which is also what can be done about it. */
export enum EEquipmentProblemKind {
  /** The configuration could not be read, so nothing is annotated. */
  CONFIG = "config",
  /** A rectangle claims part of the grid the sheet does not cover. */
  OUTSIDE = "outside",
  /** The sheet is served out of an archive, so nothing can be written to it in place. */
  ARCHIVED = "archived",
}

/** One thing worth saying about the open sheet. */
export interface IEquipmentProblem {
  kind: EEquipmentProblemKind;
  title: string;
  /** What to do about it, or the detail that makes it actionable. */
  detail: string;
}

/**
 * Everything wrong with the open sheet, worst first.
 *
 * @param metadata - The open sheet as the backend described it, or null while nothing is open.
 * @param layout - The lattice and its occupants, or null while nothing is open.
 * @returns What to list, or nothing when there is nothing to say.
 */
export function toEquipmentProblems(
  metadata: Nullable<EquipmentSpriteMetadata>,
  layout: Nullable<IEquipmentLayout>
): Array<IEquipmentProblem> {
  if (!metadata) {
    return [];
  }

  const problems: Array<IEquipmentProblem> = [];

  if (metadata.configError) {
    problems.push({
      kind: EEquipmentProblemKind.CONFIG,
      title: "The configuration was not read",
      detail: metadata.configError,
    });
  }

  const outside: ReadonlyArray<EquipmentSlotOccupant> = layout?.outside ?? [];

  if (outside.length) {
    problems.push({
      kind: EEquipmentProblemKind.OUTSIDE,
      title: `${outside.length} rectangle(s) fall outside the sheet`,
      detail: `The sheet is ${layout?.grid.sheetColumns} by ${layout?.grid.sheetRows} cells and these reach past it: ${outside
        .map((occupant: EquipmentSlotOccupant) => occupant.section)
        .join(", ")}`,
    });
  }

  if (!metadata.location.path) {
    problems.push({
      kind: EEquipmentProblemKind.ARCHIVED,
      title: "The sheet is served out of an archive",
      detail: metadata.location.writeTarget
        ? `A save would create ${metadata.location.writeTarget}, which shadows it`
        : "No mounted root can take a write, so nothing here can be saved",
    });
  }

  return problems;
}
