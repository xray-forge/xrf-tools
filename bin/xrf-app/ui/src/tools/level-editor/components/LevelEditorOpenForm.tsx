import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { StringListFormRow } from "@xrf/ui/form/string-list-row";
import { ReactElement, useCallback, useState } from "react";

import { LEVEL_SCENE_FILE_FILTERS } from "@/core/path/file-filters";
import { EToolId } from "@/core/routing/tool";
import { PickerForm } from "@/core/shell/chrome/PickerForm";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

interface ILevelEditorOpenFormProps extends BaseComponentProps {
  /** Called once a scene has been asked for, whether it opened or failed. */
  onFinished?: () => void;
}

/**
 * The way into the editor: a scene's `.level`, and where its library objects are looked for.
 */
export function LevelEditorOpenForm({
  "data-testid": dataTestId = "level-editor-open-form",
  id,
  className,
  onFinished,
}: ILevelEditorOpenFormProps): ReactElement {
  const sceneService: LevelEditorSceneService = useInjection(LevelEditorSceneService);

  const [objectsRoots, setObjectsRoots] = useState<Array<string>>([]);

  const isLoading: boolean = sceneService.scene.isLoading;

  const scene: IPathField = usePathField({
    tool: EToolId.LEVEL_EDITOR,
    id: "scene",
    title: "Select scene file",
    filters: LEVEL_SCENE_FILE_FILTERS,
    isDisabled: isLoading,
  });

  const path: Nullable<string> = scene.value;

  const onOpen = useCallback(async () => {
    if (!path) {
      return;
    }

    const roots: Array<string> = objectsRoots.map((root: string) => root.trim()).filter(Boolean);

    await sceneService.open({ objectsRoots: roots.length ? roots : null, source: { path } });

    onFinished?.();
  }, [objectsRoots, onFinished, path, sceneService]);

  return (
    <PickerForm
      data-testid={dataTestId}
      id={id}
      className={className}
      isLoading={isLoading}
      title={"Open scene"}
      description={"Reads a Level Editor scene and its part files to list and inspect them. Nothing is written."}
      error={sceneService.scene.error?.message}
      submitLabel={"Open"}
      isSubmitDisabled={!scene.isValid}
      onSubmit={onOpen}
    >
      <PathFormRow
        label={"Scene"}
        description={"The scene's .level, rawdata\\levels\\<level>.level; its parts lie beside it"}
        isDisabled={isLoading}
        field={scene}
      />

      <StringListFormRow
        label={"Objects roots"}
        description={
          "Where its library objects are looked for, in order; none for objects beside the scene's directory"
        }
        values={objectsRoots}
        addLabel={"Add objects root"}
        emptyLabel={"None: objects beside the scene's directory"}
        placeholder={"C:\\SDK\\editors\\rawdata\\objects"}
        isDisabled={isLoading}
        onChange={setObjectsRoots}
      />
    </PickerForm>
  );
}
