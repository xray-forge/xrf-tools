import { default as LayersIcon } from "@mui/icons-material/Layers";
import { Alert, Box, CircularProgress, Dialog, DialogContent } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useId } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveResolution, ArchiveResolutionSource, ArchiveUnreadSource } from "@/core/ipc/types/xrf-app";
import { DIALOG } from "@/core/theme/tokens";
import { DialogHeader } from "@/core/ui/dialog/DialogHeader";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { ArchiveResolutionSourceRow } from "./ArchiveResolutionSourceRow";
import { ArchiveResolutionUnreadRow } from "./ArchiveResolutionUnreadRow";

export interface IArchiveResolutionDialogProps extends BaseComponentProps {
  isOpen: boolean;
  onClose: () => void;
}

/**
 * Where the open subject looks for an engine path, in the order it looks.
 */
export function ArchiveResolutionDialog({
  "data-testid": dataTestId = "archive-resolution-dialog",
  id,
  className,
  isOpen,
  onClose,
}: IArchiveResolutionDialogProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const titleId: string = useId();

  const resolution: Nullable<ArchiveResolution> = archivesService.resolution.value;

  useEffect(() => {
    if (isOpen) {
      void archivesService.loadResolution();
    }
  }, [archivesService, isOpen]);

  return (
    <Dialog
      data-testid={dataTestId}
      aria-labelledby={titleId}
      id={id}
      className={className}
      fullWidth
      maxWidth={"md"}
      open={isOpen}
      onClose={onClose}
    >
      <DialogHeader
        title={"Resolution"}
        titleId={titleId}
        icon={<LayersIcon fontSize={"small"} />}
        closeLabel={"Close resolution"}
        onClose={onClose}
      />

      <DialogContent
        sx={{
          display: "flex",
          flexDirection: "column",
          gap: 2,
          height: 460,
          maxHeight: "64vh",
          paddingX: DIALOG.paddingX,
        }}
      >
        {archivesService.resolution.error ? (
          <Alert severity={"error"}>
            {`Could not describe how this is resolved: ${archivesService.resolution.error.message}`}
          </Alert>
        ) : null}

        {archivesService.resolution.isLoading ? (
          <Box sx={{ display: "flex", justifyContent: "center", paddingY: 4 }}>
            <CircularProgress size={24} />
          </Box>
        ) : null}

        {resolution ? (
          <>
            <DetailSection
              data-testid={"archive-resolution-order-section"}
              title={"Search order"}
              description={
                "Sources are searched top to bottom, and the first one holding an engine path is the copy the engine " +
                "loads. A game folder is searched in its fsgame.ltx declaration order reversed, because the engine " +
                "registers roots as declared and a later registration overwrites an earlier one."
              }
              fact={`${resolution.sources.length} source(s)`}
            >
              {resolution.sources.map((source: ArchiveResolutionSource, index: number) => (
                <ArchiveResolutionSourceRow key={`${source.path}:${source.kind}`} source={source} rank={index + 1} />
              ))}
            </DetailSection>

            {resolution.unread.length ? (
              <DetailSection
                data-testid={"archive-resolution-unread-section"}
                title={"Unread sources"}
                description={
                  "Declared here but never opened, so nothing they hold is reachable. Absent content reads exactly " +
                  "like content that was never there, which is why these are listed rather than left out."
                }
                fact={`${resolution.unread.length} source(s)`}
              >
                {resolution.unread.map((unread: ArchiveUnreadSource) => (
                  <ArchiveResolutionUnreadRow key={unread.path} source={unread} />
                ))}
              </DetailSection>
            ) : null}
          </>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}
