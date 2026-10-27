import { default as RefreshIcon } from "@mui/icons-material/Refresh";
import { Alert, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { format } from "date-fns";
import { ReactElement, useCallback } from "react";

import { RELOAD_EQUIPMENT_SPRITE_KEYBIND_COMMAND } from "@/applications/sprite-equipment-editor/commands";
import { SPRITE_EQUIPMENT_EDITOR_PANELS } from "@/applications/sprite-equipment-editor/components/panels/sprite-equipment-editor-panels";
import { SPRITE_EQUIPMENT_EDITOR_MENU } from "@/applications/sprite-equipment-editor/menus";
import {
  IOpenEquipmentSprite,
  SpriteEquipmentEditorService,
} from "@/applications/sprite-equipment-editor/services/editor";
import { EquipmentGridService } from "@/applications/sprite-equipment-editor/services/grid";
import { toAssetLocation } from "@/core/assets/lib";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarLocation, IEditorLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { useEditorBusy } from "@/core/shell/editor-lifecycle";
import { useEditorMenus, useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { IMenuContribution } from "@/core/shell/menu/lib";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { EquipmentGridOptions } from "./EquipmentGridOptions";
import { EquipmentOccupancyToggle } from "./EquipmentOccupancyToggle";
import { EquipmentRepackAction } from "./EquipmentRepackAction";
import { EquipmentSpriteEditorWorkspace } from "./EquipmentSpriteEditorWorkspace";

export function EquipmentSpriteEditor({
  "data-testid": dataTestId = "equipment-sprite-editor",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);
  const gridService: EquipmentGridService = useInjection(EquipmentGridService);
  const spriteImage: Nullable<IOpenEquipmentSprite> = spriteEquipmentService.spriteImage.value;
  const location: Nullable<IEditorLocation> = spriteImage
    ? (toAssetLocation(spriteImage.metadata.location.asset) ??
      (spriteImage.metadata.location.path ? { path: spriteImage.metadata.location.path } : null))
    : null;

  const isLoading: boolean = spriteEquipmentService.spriteImage.isLoading;
  const repackedAt: Nullable<number> = spriteEquipmentService.repackedAt;
  const error: Nullable<Error> = spriteEquipmentService.spriteImage.error;

  const onReload = useCallback(() => void spriteEquipmentService.reopenEquipmentProject(), [spriteEquipmentService]);

  // Closing does not navigate: the application shows its own picker again once nothing is open.
  const onClose = useCallback(() => spriteEquipmentService.closeEquipmentProject(), [spriteEquipmentService]);

  const outsideCount: number = gridService.layout?.outside.length ?? 0;

  useEditorPanels(() => SPRITE_EQUIPMENT_EDITOR_PANELS, []);

  useEditorMenus(
    (): Array<IMenuContribution> => [
      {
        section: SPRITE_EQUIPMENT_EDITOR_MENU.sections.sprite,
        entries: [{ kind: "command", command: RELOAD_EQUIPMENT_SPRITE_KEYBIND_COMMAND }],
      },
    ],
    []
  );

  useEditorStatus(
    spriteImage
      ? [
          `${spriteImage.image.width} x ${spriteImage.image.height}`,
          `${spriteImage.metadata.occupants.length} occupants`,
          // Only when there are any: a sheet whose configuration fits inside it should not carry a zero.
          ...(outsideCount ? [`${outsideCount} outside`] : []),
          // Where the sheet came from, said only when it is the case worth knowing about.
          ...(spriteImage.metadata.location.path ? [] : ["archived"]),
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
          subtitle={location ? <EditorToolbarLocation location={location} /> : undefined}
          actions={
            <>
              <EquipmentOccupancyToggle />

              <EquipmentGridOptions />

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
            <Typography className={"wrap-anywhere"} variant={"caption"}>
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
