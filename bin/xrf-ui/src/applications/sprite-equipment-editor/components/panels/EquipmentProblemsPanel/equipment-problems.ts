import { Nullable } from "@xrf/types";

import { EquipmentSpriteMetadata } from "@/core/ipc/types/xrf-app";
import { EquipmentSlotOccupant } from "@/core/ipc/types/xrf-texture";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";
import { IEquipmentLayout } from "@/core/sprite-equipment/lib";

/** What kind of thing is wrong, which is also what can be done about it. */
export enum EEquipmentProblemRule {
  /** The configuration could not be read, so nothing is annotated. */
  CONFIG = "config",
  /** A rectangle claims part of the grid the sheet does not cover. */
  OUTSIDE = "outside",
  /** The sheet is served out of an archive, so nothing can be written to it in place. */
  ARCHIVED = "archived",
}

/**
 * Everything worth saying about the open sheet, worst first.
 *
 * @param metadata - The open sheet as the backend described it, or null while nothing is open.
 * @param layout - The lattice and its occupants, or null while nothing is open.
 * @returns The findings to list, in the order they should be read.
 */
export function toEquipmentProblems(
  metadata: Nullable<EquipmentSpriteMetadata>,
  layout: Nullable<IEquipmentLayout>
): Array<IEditorProblem> {
  if (!metadata) {
    return [];
  }

  const problems: Array<IEditorProblem> = [];

  if (metadata.configError) {
    problems.push({
      rule: EEquipmentProblemRule.CONFIG,
      subject: metadata.open.config?.kind === "file" ? metadata.open.config.path : null,
      message: metadata.configError,
    });
  }

  const outside: ReadonlyArray<EquipmentSlotOccupant> = layout?.outside ?? [];

  if (outside.length && layout) {
    problems.push({
      rule: EEquipmentProblemRule.OUTSIDE,
      subject: `${outside.length} of ${layout.occupants.length}`,
      message:
        `The sheet covers ${layout.grid.sheetColumns} by ${layout.grid.sheetRows} cells and these reach past it: ` +
        outside.map((occupant: EquipmentSlotOccupant) => occupant.section).join(", "),
    });
  }

  if (!metadata.location.path) {
    problems.push({
      rule: EEquipmentProblemRule.ARCHIVED,
      subject: metadata.location.asset?.container.kind === "archive" ? metadata.location.asset.container.path : null,
      message: metadata.location.writeTarget
        ? `The sheet is served out of an archive. A save would create ${metadata.location.writeTarget}, which shadows it.`
        : "The sheet is served out of an archive and no mounted root can take a write, so nothing here can be saved.",
    });
  }

  return problems;
}
