import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import {
  ARCHIVE_FILTERS,
  EArchiveOpenMode,
  OPEN_MODE_DESCRIPTIONS,
  OPEN_MODE_OPTIONS,
  OPEN_MODES,
} from "@/applications/archives-explorer/components/ArchivesEditorOpenForm.utils";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { createRoot } from "@/core/assets/lib";
import { useRootProbe } from "@/core/assets/lib/use-root-probe";
import { EXrayMountMode } from "@/core/ipc/types/xrf-vfs";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { ChoiceFormRow, IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
import { inline } from "@/lib/callbacks/inline";
import { Logger, useLogger } from "@/lib/logging";
import { assertExhaustive } from "@/lib/types/exhaustive";
import { Nullable } from "@/lib/types/general";

/**
 * Opens a game folder, loose gamedata, a directory of volumes, or one volume for browsing.
 */
export function ArchivesEditorOpenForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const archivesService: ArchivesService = useInjection(ArchivesService);

  const isLoading: boolean = archivesService.subject.isLoading;

  // Keep the archive-directory default until the user chooses a mode.
  const [mode, setMode] = useRememberedValue<EArchiveOpenMode>({
    id: "mode",
    application: EApplicationId.ARCHIVES_EXPLORER,
    allowed: OPEN_MODES,
    fallback: EArchiveOpenMode.DIRECTORY,
  });

  const directory: IPathField = usePathField({
    id: "source",
    application: EApplicationId.ARCHIVES_EXPLORER,
    title: "Select archives directory",
    isDirectory: true,
    isDisabled: isLoading,
  });

  const archive: IPathField = usePathField({
    id: "archive",
    application: EApplicationId.ARCHIVES_EXPLORER,
    title: "Select archive volume",
    filters: ARCHIVE_FILTERS,
    isDisabled: isLoading,
  });

  const game: IPathField = usePathField({
    id: "game",
    application: EApplicationId.ARCHIVES_EXPLORER,
    title: "Select game folder",
    isDirectory: true,
    isDisabled: isLoading,
  });

  const gamedata: IPathField = usePathField({
    id: "gamedata",
    application: EApplicationId.ARCHIVES_EXPLORER,
    title: "Select gamedata directory",
    isDirectory: true,
    isDisabled: isLoading,
  });

  // A folder named by hand is a guess until something confirms it, and this is the mode where guessing wrong is
  // quiet: a game folder named one level too high mounts nothing and reads as an installation with no files.
  const gameFact: Nullable<string> = useRootProbe(mode === EArchiveOpenMode.GAME && !game.error ? game.value : null);

  const fields: Record<EArchiveOpenMode, IPathField> = {
    [EArchiveOpenMode.DIRECTORY]: directory,
    [EArchiveOpenMode.ARCHIVE]: archive,
    [EArchiveOpenMode.GAME]: game,
    [EArchiveOpenMode.GAMEDATA]: gamedata,
  };
  const field: IPathField = fields[mode];

  const onOpen = useCallback(() => {
    if (!field.value) {
      log.info("Cannot parse archives project without path");

      return;
    }

    if (mode === EArchiveOpenMode.GAME) {
      archivesService.openWorld({ asset: null, roots: [createRoot(field.value)] });
    } else if (mode === EArchiveOpenMode.GAMEDATA) {
      archivesService.openWorld({ asset: null, roots: [createRoot(field.value, EXrayMountMode.DIRECTORY)] });
    } else {
      archivesService.openVolumes(field.value);
    }
  }, [archivesService, field.value, log, mode]);

  return (
    <PickerForm
      isLoading={isLoading}
      title={"Open game archives"}
      description={OPEN_MODE_DESCRIPTIONS[mode]}
      error={archivesService.subject.error ? archivesService.subject.error.message : undefined}
      submitLabel={"Open"}
      isSubmitDisabled={!field.isValid}
      onSubmit={onOpen}
    >
      <ChoiceFormRow
        label={"Open"}
        description={"A game folder, loose gamedata, a directory of volumes, or one archive"}
        options={OPEN_MODE_OPTIONS}
        value={mode}
        isDisabled={isLoading}
        onChange={setMode}
      />

      {inline(() => {
        switch (mode) {
          case EArchiveOpenMode.GAME:
            return (
              <PathFormRow
                isDisabled={isLoading}
                label={"Game folder"}
                description={"Folder holding fsgame.ltx, or a game data tree on its own"}
                fact={gameFact}
                field={game}
              />
            );

          case EArchiveOpenMode.GAMEDATA:
            return (
              <PathFormRow
                isDisabled={isLoading}
                label={"Gamedata directory"}
                description={"Folder containing loose game files, such as configs, textures and meshes"}
                field={gamedata}
              />
            );

          case EArchiveOpenMode.DIRECTORY:
            return (
              <PathFormRow
                isDisabled={isLoading}
                label={"Archives directory"}
                description={"Directory holding the packed game archives"}
                field={directory}
              />
            );

          case EArchiveOpenMode.ARCHIVE:
            return (
              <PathFormRow
                isDisabled={isLoading}
                label={"Archive volume"}
                description={"Single packed archive to open"}
                field={archive}
              />
            );

          default:
            return assertExhaustive(mode);
        }
      })}
    </PickerForm>
  );
}
