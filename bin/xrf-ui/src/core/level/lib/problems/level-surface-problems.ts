import { Nullable } from "@xrf/types";

import { EXraySurfaceDeclaration, XraySurfaceDeclaration, XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { ELevelProblemRule } from "@/core/level/lib/problems/level-problem-rule";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

/**
 * The shader table entries the library answered nothing useful for.
 *
 * @param surfaces - The level's resolved shader table, in its own order.
 * @returns The rows, each naming the entry by its index.
 */
export function listSurfaceProblems(surfaces: ReadonlyArray<XraySurfaceDescriptor>): Array<IEditorProblem> {
  const problems: Array<IEditorProblem> = [];

  surfaces.forEach((surface: XraySurfaceDescriptor, index: number) => {
    const message: Nullable<string> = describeDeclaration(surface.declaration);

    if (message) {
      problems.push({ message, rule: ELevelProblemRule.SURFACE, subject: `shader table entry ${index}` });
    }
  });

  return problems;
}

/** What is wrong with one declaration, or null for one that is not wrong. */
function describeDeclaration(declaration: XraySurfaceDeclaration): Nullable<string> {
  switch (declaration.kind) {
    case EXraySurfaceDeclaration.NO_LIBRARY:
      return "No shaders.xr in any searched root, so no surface of this level could be described";

    case EXraySurfaceDeclaration.UNREADABLE:
      return `The shader library could not be read: ${declaration.reason}`;

    case EXraySurfaceDeclaration.UNDEFINED:
      return "The shader library holds no blender of that name, so the surface is drawn opaque";

    case EXraySurfaceDeclaration.UNMODELLED:
      return `Blender class '${declaration.class}' is not modelled, so the surface is drawn opaque`;

    default:
      return null;
  }
}
