import { default as RefreshIcon } from "@mui/icons-material/Refresh";
import { Alert, IconButton, Tooltip, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { format } from "date-fns";
import { ReactElement, useCallback, useEffect } from "react";

import { EquipmentRepackAction } from "@/applications/sprite-equipment-editor/components/equipment-editor/EquipmentRepackAction";
import { EquipmentSpriteEditorWorkspace } from "@/applications/sprite-equipment-editor/components/equipment-editor/EquipmentSpriteEditorWorkspace";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorBusy } from "@/core/shell/EditorBusyContext";
import { useEditorStatus } from "@/core/shell/EditorStatusContext";
import { IEquipmentPngDescriptor, SpriteEquipmentService } from "@/core/sprite-equipment";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function EquipmentSpriteEditor(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const spriteEquipmentService: SpriteEquipmentService = useInjection(SpriteEquipmentService);
  const spriteImage: Nullable<IEquipmentPngDescriptor> = spriteEquipmentService.spriteImage.value;

  const isLoading: boolean = spriteEquipmentService.spriteImage.isLoading;
  const repackedAt: Nullable<number> = spriteEquipmentService.repackedAt;
  const error: Nullable<Error> = spriteEquipmentService.spriteImage.error;

  const onReload = useCallback(async () => {
    try {
      await spriteEquipmentService.reopenEquipmentProject();
    } catch (error) {
      // Published as the sprite failure and rendered by the menu. Logged here for the stack.
      log.error("Failed to reload DDS:", error);
    }
  }, [log, spriteEquipmentService]);

  // Closing does not navigate: the application shows its own picker again once nothing is open.
  const onClose = useCallback(() => spriteEquipmentService.closeEquipmentProject(), [spriteEquipmentService]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent): void {
      if (event.key === "F5" && !isLoading) {
        event.preventDefault();
        void onReload();
      }
    }

    window.addEventListener("keydown", onKeyDown);

    return () => window.removeEventListener("keydown", onKeyDown);
  }, [isLoading, onReload]);

  useEditorStatus(
    spriteImage
      ? [
          `${spriteImage.image.width} x ${spriteImage.image.height}`,
          `${spriteImage.descriptors.length} descriptors`,
          ...(repackedAt ? [`Repacked ${format(repackedAt, "HH:mm")}`] : []),
        ]
      : []
  );

  useEditorBusy(isLoading);

  return (
    <EditorLayout
      toolbar={
        <EditorToolbar
          subtitle={spriteImage?.path}
          actions={
            <>
              <EquipmentRepackAction />

              <Tooltip describeChild title={"Reload sprite (F5)"}>
                <span>
                  <IconButton aria-label={"Reload sprite"} color={"inherit"} disabled={isLoading} onClick={onReload}>
                    <RefreshIcon fontSize={"small"} />
                  </IconButton>
                </span>
              </Tooltip>
            </>
          }
          onBack={onClose}
        />
      }
      banner={
        error ? (
          <Alert severity={"error"} variant={"outlined"} onClose={spriteEquipmentService.clearSpriteError}>
            <Typography variant={"caption"} sx={{ wordBreak: "break-word" }}>
              {String(error)}
            </Typography>
          </Alert>
        ) : null
      }
    >
      <EquipmentSpriteEditorWorkspace />
    </EditorLayout>
  );
}
