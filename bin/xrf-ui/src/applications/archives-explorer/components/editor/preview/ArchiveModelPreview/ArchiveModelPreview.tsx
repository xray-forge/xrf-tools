import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useEffect } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { getSubjectRoots } from "@/core/archive/lib";
import { ArchiveSubject } from "@/core/ipc/types/xrf-app";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { VisualPreviewViewport } from "@/core/visuals/components/preview";
import { IOpenVisual, VisualLoadService } from "@/core/visuals/services";
import { AsyncState } from "@/lib/async-state";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
      void loadService.load({ kind: "asset", logicalPath: name }, getSubjectRoots(subject));
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
