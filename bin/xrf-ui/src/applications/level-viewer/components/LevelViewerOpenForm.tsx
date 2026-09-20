import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { createRoots } from "@/core/assets/lib";
import { LevelEntry } from "@/core/ipc/types/xrf-app";
import { LevelListService, LevelLoadService } from "@/core/level/services";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { ChoiceListFormRow, IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelViewerOpenFormProps extends BaseComponentProps {
  /** Called once a level has been opened. */
  onFinished?: () => void;
}

/**
 * The way into the viewer: point at an installation, then pick one of its levels.
 */
export function LevelViewerOpenForm({
  "data-testid": dataTestId = "level-viewer-open-form",
  id,
  className,
  onFinished,
}: ILevelViewerOpenFormProps): ReactElement {
  const listService: LevelListService = useInjection(LevelListService);
  const loadService: LevelLoadService = useInjection(LevelLoadService);

  const [selected, setSelected] = useState<string>("");

  const isLoading: boolean = listService.levels.isLoading || loadService.level.isLoading;

  const root: IPathField = usePathField({
    application: EApplicationId.LEVEL_VIEWER,
    id: "root",
    isDirectory: true,
    isDisabled: isLoading,
    title: "Select gamedata or installation directory",
  });

  const levels: Array<LevelEntry> = listService.drawable;

  const options = useMemo(
    () => levels.map((entry: LevelEntry) => ({ label: entry.name, value: entry.logicalPath })),
    [levels]
  );

  const isListed: boolean = Boolean(listService.levels.value);

  const onList = useCallback(async () => {
    if (!root.value) {
      return;
    }

    await listService.list(createRoots([root.value]));

    // Preselected, so listing an installation leaves a choice that can be submitted rather than an empty one.
    setSelected(listService.drawable[0]?.logicalPath ?? "");
  }, [listService, root.value]);

  const onOpen = useCallback(async () => {
    if (!selected || !root.value) {
      return;
    }

    await loadService.load({ kind: "asset", logicalPath: selected }, createRoots([root.value]));

    onFinished?.();
  }, [loadService, onFinished, root.value, selected]);

  return (
    <PickerForm
      data-testid={dataTestId}
      id={id}
      className={className}
      isLoading={isLoading}
      title={"Open compiled level"}
      description={
        isListed
          ? "Opens the level's structure and streams its geometry as you fly. Nothing is written."
          : "Lists the levels the roots hold, archives included. Nothing is written."
      }
      error={listService.levels.error?.message ?? loadService.level.error?.message}
      submitLabel={isListed ? "Open" : "List levels"}
      isSubmitDisabled={isListed ? !selected : !root.isValid}
      onSubmit={isListed ? onOpen : onList}
    >
      <PathFormRow
        label={"Game root"}
        description={"Installation or gamedata directory holding the levels"}
        isDisabled={isLoading}
        field={root}
      />

      {isListed ? (
        <ChoiceListFormRow
          label={"Level"}
          description={
            options.length ? "Compiled levels these roots hold" : "These roots hold no level with render geometry"
          }
          options={options}
          value={selected}
          isDisabled={isLoading || !options.length}
          onChange={setSelected}
        />
      ) : null}
    </PickerForm>
  );
}
