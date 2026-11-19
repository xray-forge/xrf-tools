import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useCallback, useState } from "react";

import { SessionTool } from "@/core/session/components/SessionTool";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

import { LevelEditorOpenForm } from "./components/LevelEditorOpenForm";
import { LevelEditorWorkspace } from "./components/LevelEditorWorkspace";

/**
 * Opens a Level Editor scene to inspect, read-only. A failed open shows why in the viewport, with a retry, and the open
 * form one step back.
 */
export function LevelEditorTool({
  "data-testid": dataTestId = "level-editor-tool",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const sceneService: LevelEditorSceneService = useInjection(LevelEditorSceneService);

  const [isFormOpen, setFormOpen] = useState<boolean>(false);

  const path: Nullable<string> = sceneService.scene.value?.source.path ?? null;

  const onBack = useCallback(() => setFormOpen(true), []);

  const onFinished = useCallback(() => setFormOpen(false), []);

  const onClose = useCallback(() => void sceneService.close(), [sceneService]);

  const onRetry = useCallback(() => void sceneService.retry(), [sceneService]);

  return (
    <SessionTool
      root={sceneService.scene}
      isOpen={!isFormOpen && (path !== null || sceneService.scene.isLoading || sceneService.scene.state.isFailed)}
      openForm={<LevelEditorOpenForm onFinished={onFinished} />}
      workspace={
        <LevelEditorWorkspace
          data-testid={dataTestId}
          id={id}
          className={className}
          name={path}
          pending={sceneService.opening?.path ?? null}
          isLoading={sceneService.scene.isLoading}
          isStale={sceneService.isStale}
          error={sceneService.scene.error?.message}
          onRetry={onRetry}
          onBack={onBack}
          onClose={onClose}
        />
      }
    />
  );
}
