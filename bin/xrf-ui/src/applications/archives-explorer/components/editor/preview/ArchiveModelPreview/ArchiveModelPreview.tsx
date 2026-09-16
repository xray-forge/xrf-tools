import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { getSubjectRoots } from "@/core/archive/lib";
import { ArchiveSubject } from "@/core/ipc/types/xrf-app";
import { mergeSx } from "@/core/theme/merge-sx";
import { getSurfaceSx } from "@/core/theme/surface";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { VisualPreviewViewport } from "@/core/visuals/components/preview";
import { DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS } from "@/core/visuals/components/scene";
import { IOpenVisual, VisualLoadService } from "@/core/visuals/services";
import { AsyncState } from "@/lib/async-state";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

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
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ position: "relative", display: "flex", flexGrow: 1, minWidth: 0, minHeight: 0, overflow: "hidden" }}
    >
      <VisualPreviewViewport
        detail={0}
        model={visual.value?.views ?? null}
        options={DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS}
        textures={loadService.textures}
        bumps={loadService.bumps}
      />

      {visual.isLoading ? (
        <Box sx={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center" }}>
          <DelayedProgress />
        </Box>
      ) : null}

      {!visual.value && !visual.isLoading ? (
        <Box sx={mergeSx(getSurfaceSx("content"), { position: "absolute", inset: 0, display: "flex" })}>
          <EmptyState
            title={visual.error ? "Could not read this model" : "No model to show"}
            description={visual.error?.message ?? name}
          />
        </Box>
      ) : null}
    </Box>
  );
}
