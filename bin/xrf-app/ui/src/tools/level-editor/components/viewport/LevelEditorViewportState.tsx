import { assertExhaustive, Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EmptyState } from "@xrf/ui/layout/empty-state";
import { ErrorState } from "@xrf/ui/layout/error-state";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { ELevelEditorSceneBlank } from "@/tools/level-editor/lib/render/level-editor-scene-blank";

/**
 * @param blank - Why an open scene draws nothing.
 * @returns What the viewport says of it.
 */
function describeSceneBlank(blank: ELevelEditorSceneBlank): string {
  switch (blank) {
    case ELevelEditorSceneBlank.EMPTY:
      return "This scene holds no objects: none of its tools places anything.";

    case ELevelEditorSceneBlank.NOTHING_FOUND:
      return "None of this scene's library objects was found. Check the objects roots; Problems lists each.";

    case ELevelEditorSceneBlank.NOTHING_DRAWN:
      return (
        "This scene has no scene objects, and what it holds draws no helper: its sectors, groups and the rest are " +
        "listed in the Outliner."
      );

    case ELevelEditorSceneBlank.HELPERS_HIDDEN:
      return "This scene has no scene objects, and its helpers are switched off. Turn them on in Show.";

    default:
      return assertExhaustive(blank);
  }
}

interface ILevelEditorViewportStateProps extends BaseComponentProps {
  /** Why the last open failed, or absent where nothing went wrong. */
  error?: string;
  /** Why the open scene draws nothing, or null for no scene open. */
  blank?: Nullable<ELevelEditorSceneBlank>;
  onRetry?: () => void;
}

/**
 * What covers the viewport while it draws no scene: why the last open failed, with a retry, why the scene open draws
 * nothing, or that none is open.
 */
export function LevelEditorViewportState({
  "data-testid": dataTestId = "level-editor-viewport-state",
  id,
  className,
  error,
  blank = null,
  onRetry,
}: ILevelEditorViewportStateProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("absolute inset-0 flex surface-content", className)}>
      {error ? (
        <ErrorState title={"Could not open this scene"} description={error} onRetry={onRetry} />
      ) : blank ? (
        <EmptyState title={"Nothing to draw"} description={describeSceneBlank(blank)} />
      ) : (
        <EmptyState title={"No scene open"} description={"Open a scene's .level to draw it."} />
      )}
    </div>
  );
}
