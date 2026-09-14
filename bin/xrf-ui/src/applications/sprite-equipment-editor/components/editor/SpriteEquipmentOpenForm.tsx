import { TextField } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { SpriteEquipmentEditorService } from "@/applications/sprite-equipment-editor/services/editor";
import { createRoots } from "@/core/assets/lib";
import { EquipmentSpriteOpen } from "@/core/ipc/types/xrf-app";
import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { ConfigsDialectFormRow } from "@/core/ltx/components/configs-dialect/ConfigsDialectFormRow";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { ChoiceFormRow, FormRow, IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

import {
  DEFAULT_SHEET_REFERENCE,
  EEquipmentOpenMode,
  GAME_CONFIG_PATH,
  OPEN_MODE_DESCRIPTIONS,
  OPEN_MODE_OPTIONS,
  OPEN_MODES,
} from "./SpriteEquipmentOpenForm.utils";

export function SpriteEquipmentOpenForm({
  "data-testid": dataTestId = "sprite-equipment-open-form",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);

  const isLoading: boolean = spriteEquipmentService.spriteImage.isLoading;

  const [mode, setMode] = useRememberedValue<EEquipmentOpenMode>({
    allowed: OPEN_MODES,
    application: EApplicationId.SPRITE_EQUIPMENT_EDITOR,
    fallback: EEquipmentOpenMode.GAME,
    id: "open-mode",
  });

  const game: IPathField = usePathField({
    application: EApplicationId.SPRITE_EQUIPMENT_EDITOR,
    id: "game",
    title: "Select a game installation or game data directory",
    isDirectory: true,
    isDisabled: isLoading,
  });

  const sprite: IPathField = usePathField({
    application: EApplicationId.SPRITE_EQUIPMENT_EDITOR,
    id: "sprite",
    title: "Select equipment sprite",
    filters: [{ name: "dds", extensions: [EXrayExtension.DDS] }],
    isDisabled: isLoading,
  });

  const systemLtx: IPathField = usePathField({
    application: EApplicationId.SPRITE_EQUIPMENT_EDITOR,
    id: "system-ltx",
    title: "Select system.ltx",
    filters: [{ name: "ltx", extensions: [EXrayExtension.LTX] }],
    isDisabled: isLoading,
  });

  // todo: Offer the sheets the tree actually holds, which needs a backend listing of `textures\ui\ui_icon_equipment*`.
  const [reference, setReference] = useState<string>(DEFAULT_SHEET_REFERENCE);

  // Opt-in rather than detected: a patched Anomaly tree and a vanilla one look alike, and resolving one under the
  // other's rules answers the wrong slot occupants rather than failing.
  const [isDltx, setDltx] = useState<boolean>(false);

  const isGame: boolean = mode === EEquipmentOpenMode.GAME;

  /**
   * What the two modes each ask the backend for, or null while a field they need is still empty.
   *
   * Also what says whether the form can be submitted, rather than a second reading of the same fields: a validity
   * rule that agrees with the request only by coincidence is a submit button that enables for a request nobody can
   * build.
   */
  const open: Nullable<EquipmentSpriteOpen> = useMemo(() => {
    if (isGame) {
      return game.isValid && game.value && reference.trim()
        ? {
            roots: createRoots([game.value]),
            sheet: { kind: "asset", reference: reference.trim() },
            config: { kind: "asset", logicalPath: GAME_CONFIG_PATH },
            isDltx,
          }
        : null;
    }

    return sprite.isValid && systemLtx.isValid && sprite.value && systemLtx.value
      ? {
          // A loose sheet is read by path and needs no game behind it; the configuration beside it resolves from its
          // own directory, which is what carries any `mod_*.ltx` sitting there.
          roots: createRoots([]),
          sheet: { kind: "file", path: sprite.value },
          config: { kind: "file", path: systemLtx.value },
          isDltx,
        }
      : null;
  }, [
    isDltx,
    isGame,
    reference,
    sprite.isValid,
    sprite.value,
    systemLtx.isValid,
    systemLtx.value,
    game.isValid,
    game.value,
  ]);

  const onOpen = useCallback(() => {
    if (open) {
      spriteEquipmentService.openEquipmentProject(open);
    } else {
      log.info("Cannot open equipment editor without every path");
    }
  }, [log, open, spriteEquipmentService]);

  return (
    <PickerForm
      data-testid={dataTestId}
      id={id}
      className={className}
      isLoading={isLoading}
      isSubmitDisabled={!open}
      title={"Open equipment sprite"}
      description={OPEN_MODE_DESCRIPTIONS[mode]}
      error={spriteEquipmentService.spriteImage.error ? String(spriteEquipmentService.spriteImage.error) : undefined}
      submitLabel={"Open"}
      onSubmit={onOpen}
    >
      <ChoiceFormRow
        label={"Open"}
        description={"The game as the engine mounts it, or one sheet and the configuration beside it"}
        options={OPEN_MODE_OPTIONS}
        value={mode}
        isDisabled={isLoading}
        onChange={setMode}
      />

      {isGame ? (
        <>
          <PathFormRow
            isDisabled={isLoading}
            label={"Game folder"}
            description={"A game installation or a gamedata directory, searched for both the sheet and the configs"}
            field={game}
          />

          <FormRow label={"Sheet"} description={"Engine reference of the sheet to open"}>
            <TextField
              size={"small"}
              fullWidth={true}
              disabled={isLoading}
              value={reference}
              onChange={(event) => setReference(event.target.value)}
            />
          </FormRow>
        </>
      ) : (
        <>
          <PathFormRow
            isDisabled={isLoading}
            label={"Equipment sprite"}
            description={"The packed *.dds holding the inventory icons"}
            field={sprite}
          />

          <PathFormRow
            isDisabled={isLoading}
            label={"System configuration"}
            description={"The system.ltx that names the icons"}
            field={systemLtx}
          />
        </>
      )}

      <ConfigsDialectFormRow isDltx={isDltx} isDisabled={isLoading} onChange={setDltx} />
    </PickerForm>
  );
}
