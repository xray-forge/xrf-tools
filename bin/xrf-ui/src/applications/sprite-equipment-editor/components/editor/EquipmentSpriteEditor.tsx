import { default as RefreshIcon } from "@mui/icons-material/Refresh";
import { Alert, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { format } from "date-fns";
import { ReactElement, useCallback, useEffect } from "react";

import {
  IEquipmentPngDescriptor,
  SpriteEquipmentEditorService,
} from "@/applications/sprite-equipment-editor/services/editor";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorBusy } from "@/core/shell/editor-lifecycle";
import { useEditorStatus } from "@/core/shell/editor-shell";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

import { EquipmentRepackAction } from "./EquipmentRepackAction";
import { EquipmentSpriteEditorWorkspace } from "./EquipmentSpriteEditorWorkspace";

export function EquipmentSpriteEditor({
  "data-testid": dataTestId = "equipment-sprite-editor",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);
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
      data-testid={dataTestId}
      id={id}
      className={className}
      toolbar={
        <EditorToolbar
          subtitle={spriteImage?.path}
          actions={
            <>
              <EquipmentRepackAction />

              <EditorIconAction
                label={"Reload sprite"}
                description={"Reload sprite (F5)"}
                icon={<RefreshIcon />}
                isDisabled={isLoading}
                onClick={onReload}
              />
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
