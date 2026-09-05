import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useState } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { createArchiveRoots } from "@/core/archive";
import { ArchiveProject } from "@/core/bindings/types/xrf-archive";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { VisualPreviewViewport } from "@/core/visuals/components/preview";
import { DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS } from "@/core/visuals/components/scene";
import { IOpenVisual, VisualLoadService } from "@/core/visuals/services";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Loadable } from "@/lib/loadable";
import { Nullable } from "@/lib/types/general";

export interface IArchiveModelPreviewProps extends BaseComponentProps {
  /** Entry name as the archive records it, which is also its engine identity. */
  name: string;
}

/**
 * Shows an archived model, read out of the volumes it sits in.
 */
export function ArchiveModelPreview({
  "data-testid": dataTestId = "archive-model-preview",
  id,
  className,
  name,
}: IArchiveModelPreviewProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const loadService: VisualLoadService = useInjection(VisualLoadService);

  const project: Nullable<ArchiveProject> = archivesService.project.value;
  const visual: Loadable<Nullable<IOpenVisual>> = loadService.visual;

  const [cameraResetToken, setCameraResetToken] = useState(0);

  useEffect(() => {
    if (project) {
      void loadService.load({ kind: "asset", logicalPath: name }, createArchiveRoots(project));
    }

    return () => loadService.clear();
  }, [loadService, name, project]);

  // Refit once the model is on screen. The scene fits its camera when the geometry lands, but this viewport mounts with
  // the selection rather than with the application, so at that moment the panel is still taking its width - and a fit
  // measured against the wrong aspect leaves the model filling the frame.
  useEffect(() => {
    if (visual.value) {
      setCameraResetToken((it) => it + 1);
    }
  }, [visual.value]);

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
        cameraResetToken={cameraResetToken}
        textures={loadService.textures}
        bumps={loadService.bumps}
      />

      {visual.isLoading ? (
        <Box sx={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center" }}>
          <DelayedProgress />
        </Box>
      ) : null}

      {!visual.value && !visual.isLoading ? (
        <Box sx={{ position: "absolute", inset: 0, display: "flex", backgroundColor: "background.default" }}>
          <EmptyState
            title={visual.error ? "Could not read this model" : "No model to show"}
            description={visual.error?.message ?? name}
          />
        </Box>
      ) : null}
    </Box>
  );
}
