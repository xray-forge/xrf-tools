import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { LevelPreviewLayout } from "@/core/level/components/preview/LevelPreviewLayout";
import { LevelLoadService } from "@/core/level/services";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { LevelViewerOpenForm } from "./components/LevelViewerOpenForm";

/**
 * Fly a compiled level, streaming its sectors as the camera reaches them.
 */
export function LevelViewerApplication({
  "data-testid": dataTestId = "level-viewer-application",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);

  const [isPickerOpen, setPickerOpen] = useState(false);

  const description: Nullable<SelectedLevelDescription> = loadService.level.value?.selected.value ?? null;

  const onDescribeLevelName = useCallback((description: SelectedLevelDescription) => {
    return description.source.kind === "directory" ? description.source.path : description.source.logicalPath;
  }, []);

  const onBack = useCallback(() => setPickerOpen(true), []);

  const onFinished = useCallback(() => setPickerOpen(false), []);

  const onDeselect = useCallback(() => void loadService.close(), [loadService]);

  if (!loadService.isReady) {
    return <ApplicationLoader />;
  }

  if (isPickerOpen || (!description && !loadService.level.isLoading)) {
    return <LevelViewerOpenForm onFinished={onFinished} />;
  }

  return (
    <LevelPreviewLayout
      data-testid={dataTestId}
      id={id}
      className={className}
      name={description ? onDescribeLevelName(description) : null}
      streaming={loadService.streaming}
      isLoading={loadService.level.isLoading}
      error={loadService.level.error?.message}
      onBack={onBack}
      onDeselect={onDeselect}
    />
  );
}
