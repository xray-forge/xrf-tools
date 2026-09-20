import { EXraySurfaceDeclaration, XraySurfaceDeclaration, XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { EVisualSkipCause, SectorSkip } from "@/core/ipc/types/xrf-visual";
import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { ILevelTextureProblem } from "@/core/level/lib/level-texture-set";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

/** What a level's problems are grouped by, which is the stage of the open each one comes from. */
export enum ELevelProblemRule {
  /** A texture reference the roots could not answer properly. */
  TEXTURE = "texture",
  /** A shader table entry the library could not be read for, so the surface is drawn as an unresolved one. */
  SURFACE = "surface",
  /** A drawable the packer could not read, so nothing of it is drawn at all. */
  DRAWABLE = "drawable",
}

/**
 * Everything the viewer could not draw as the level asked, as rows.
 *
 * @param textures - What the texture set has to say, which is a reference and a reason.
 * @param surfaces - The level's resolved shader table, in its own order.
 * @param sectors - The sectors currently resident, whose packs carry what they could not read.
 * @returns The rows, textures first, then surfaces, then drawables.
 */
export function listLevelProblems(
  textures: ReadonlyArray<ILevelTextureProblem>,
  surfaces: ReadonlyArray<XraySurfaceDescriptor>,
  sectors: ReadonlyMap<number, ILoadedSector>
): Array<IEditorProblem> {
  return [...toTextureProblems(textures), ...toSurfaceProblems(surfaces), ...toDrawableProblems(sectors)];
}

function toTextureProblems(textures: ReadonlyArray<ILevelTextureProblem>): Array<IEditorProblem> {
  return textures.map((problem: ILevelTextureProblem) => ({
    message: problem.reason,
    rule: ELevelProblemRule.TEXTURE,
    subject: problem.reference,
  }));
}

/**
 * The table entries the library answered nothing useful for.
 *
 * An undeclared entry is not one of them: a level's table carries entries that name no shader, and holding their
 * place is what keeps every later entry on the right surface.
 */
function toSurfaceProblems(surfaces: ReadonlyArray<XraySurfaceDescriptor>): Array<IEditorProblem> {
  const problems: Array<IEditorProblem> = [];

  surfaces.forEach((surface: XraySurfaceDescriptor, index: number) => {
    const message: string | null = describeDeclaration(surface.declaration);

    if (message) {
      problems.push({ message, rule: ELevelProblemRule.SURFACE, subject: `shader table entry ${index}` });
    }
  });

  return problems;
}

function describeDeclaration(declaration: XraySurfaceDeclaration): string | null {
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

/** What each resident sector could not read, which is geometry that is simply absent from the picture. */
function toDrawableProblems(sectors: ReadonlyMap<number, ILoadedSector>): Array<IEditorProblem> {
  const problems: Array<IEditorProblem> = [];

  for (const [sector, loaded] of sectors) {
    for (const skip of loaded.views.skipped) {
      problems.push({
        message: `${describeCause(skip)}: ${skip.reason}`,
        rule: ELevelProblemRule.DRAWABLE,
        subject: `sector ${sector}, visual ${skip.drawable}`,
      });
    }
  }

  return problems;
}

function describeCause(skip: SectorSkip): string {
  return skip.cause === EVisualSkipCause.UNSUPPORTED ? "Stored in a form the reader does not model" : "Malformed";
}
