import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { ArchiveSubject } from "@xrf/ipc/types/xrf-app-archives";
import { EAssetSource } from "@xrf/ipc/types/xrf-app-core";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DelayedProgress } from "@xrf/ui/layout/delayed-progress";
import { EmptyState } from "@xrf/ui/layout/empty-state";
import { cn } from "@xrf/ui/theme";
import { ReactElement, useEffect } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { VisualPreviewViewport } from "@/core/visuals/components/preview";
import { IOpenVisual, VisualLoadService } from "@/core/visuals/services";

interface IArchiveModelPreviewProps extends BaseComponentProps {
  name: string;
}

/**
 * Shows a model, read out of whichever tree the explorer has open.
 */
export function ArchiveModelPreview({
  "data-testid": dataTestId = "archive-model-preview",
  id,
  className,
  name,
}: IArchiveModelPreviewProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const loadService: VisualLoadService = useInjection(VisualLoadService);

  const subject: Nullable<ArchiveSubject> = archivesService.subject.value;
  const visual: AsyncState<Nullable<IOpenVisual>> = loadService.visual;

  useEffect(() => {
    if (subject) {
      void loadService.load({ kind: EAssetSource.ASSET, logicalPath: name }, subject.roots);
    }

    return () => loadService.clear();
  }, [loadService, name, subject]);

  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("relative flex min-h-0 min-w-0 grow overflow-hidden", className)}
    >
      <VisualPreviewViewport />

      {visual.isLoading ? (
        <div className={"absolute inset-0 flex items-center justify-center"}>
          <DelayedProgress />
        </div>
      ) : null}

      {!visual.value && !visual.isLoading ? (
        <div className={"absolute inset-0 flex surface-content"}>
          <EmptyState
            title={visual.error ? "Could not read this model" : "No model to show"}
            description={visual.error?.message ?? name}
          />
        </div>
      ) : null}
    </div>
  );
}
