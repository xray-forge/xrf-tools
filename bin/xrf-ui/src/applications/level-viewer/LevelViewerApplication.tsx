import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useState } from "react";

import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { describeLevelSource } from "@/core/level/lib/source/level-source";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { LevelViewerLayout } from "./components/LevelViewerLayout";
import { LevelViewerOpenForm } from "./components/LevelViewerOpenForm";

/**
 * Fly a compiled level, shown once every sector of it is read. An open that fails shows why over the empty viewport,
 * with a retry, the picker one step back.
 */
export function LevelViewerApplication({
  "data-testid": dataTestId = "level-viewer-application",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);

  const [isPickerOpen, setPickerOpen] = useState(false);

  const description: Nullable<SelectedLevelDescription> = loadService.level.value?.selected.value ?? null;

  const onBack = useCallback(() => setPickerOpen(true), []);

  const onFinished = useCallback(() => setPickerOpen(false), []);

  const onDeselect = useCallback(() => void loadService.close(), [loadService]);

  const onRetry = useCallback(() => void loadService.retry(), [loadService]);

  if (!loadService.isReady) {
    return <ApplicationLoader />;
  }

  if (isPickerOpen || (!description && !loadService.level.isLoading && !loadService.level.isFailed)) {
    return <LevelViewerOpenForm onFinished={onFinished} />;
  }

  return (
    <LevelViewerLayout
      data-testid={dataTestId}
      id={id}
      className={className}
      name={description ? describeLevelSource(description.source) : null}
      pending={loadService.opening ? describeLevelSource(loadService.opening) : null}
      isLoading={loadService.level.isLoading}
      error={loadService.level.error?.message}
      onRetry={onRetry}
      onBack={onBack}
      onDeselect={onDeselect}
    />
  );
}
