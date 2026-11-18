import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme";
import { ReactElement, ReactNode } from "react";

import { LevelPreviewCover } from "@/core/level/components/preview/LevelPreviewCover";
import { RenderSurface } from "@/core/render/components/RenderSurface";
import { LevelEditorViewportState } from "@/tools/level-editor/components/viewport/LevelEditorViewportState";
import {
  ELevelEditorSceneBlank,
  toLevelEditorSceneBlank,
} from "@/tools/level-editor/lib/render/level-editor-scene-blank";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";
import { LevelEditorRenderService } from "@/tools/level-editor/services/level-editor-render";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

interface ILevelEditorViewportProps extends BaseComponentProps {
  /** Whether a scene is open, which chooses its cover over why the last open failed; the surface draws regardless. */
  isOpen?: boolean;
  /** Whether a scene is being opened. */
  isLoading?: boolean;
  /** Why the last open failed, shown with `onRetry` while no scene is open. */
  error?: string;
  /** Draws the native viewport, for a surface with no GPU to give one - a test. */
  renderViewport?: (props: BaseComponentProps) => ReactNode;
  onRetry?: () => void;
}

/**
 * Where the open scene is drawn: the native surface, covered while the scene opens and until every object it draws is
 * resident, the cover saying why where the renderer cannot draw it; why the scene draws nothing once its objects,
 * drawing and helpers are read and it draws neither an object nor a helper, its helpers switched off included; and
 * why the open failed while none is open.
 */
export function LevelEditorViewport({
  "data-testid": dataTestId = "level-editor-viewport",
  id,
  className,
  isOpen = false,
  isLoading = false,
  error,
  renderViewport,
  onRetry,
}: ILevelEditorViewportProps): ReactElement {
  const helpersService: LevelEditorHelpersService = useInjection(LevelEditorHelpersService);
  const renderService: LevelEditorRenderService = useInjection(LevelEditorRenderService);
  const sceneService: LevelEditorSceneService = useInjection(LevelEditorSceneService);

  const blank: Nullable<ELevelEditorSceneBlank> = toLevelEditorSceneBlank(
    sceneService.objects.value,
    sceneService.drawing.value,
    sceneService.helpers.value,
    helpersService.helpers
  );

  return (
    <div data-testid={dataTestId} id={id} className={cn("relative flex min-h-0 min-w-0 flex-1", className)}>
      {renderViewport ? renderViewport({}) : <RenderSurface host={renderService} />}

      {isOpen && !isLoading && blank ? <LevelEditorViewportState blank={blank} /> : null}

      {isOpen || isLoading ? (
        <LevelPreviewCover
          host={renderService}
          subject={"scene"}
          rest={"Compiling the scene's objects"}
          isLoading={isLoading}
        />
      ) : (
        <LevelEditorViewportState error={error} onRetry={onRetry} />
      )}
    </div>
  );
}
