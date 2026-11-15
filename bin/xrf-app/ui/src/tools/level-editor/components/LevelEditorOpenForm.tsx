import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { StringListFormRow } from "@xrf/ui/form/string-list-row";
import { ReactElement, useCallback, useState } from "react";

import { createRoots } from "@/core/assets/lib/roots";
import { EngineChoiceFormRow } from "@/core/engine-target/components/EngineChoiceFormRow";
import { IEngineChoice, useEngineChoice } from "@/core/engine-target/lib/use-engine-choice";
import { ConfigsDialectFormRow } from "@/core/ltx/components/ConfigsDialectFormRow";
import { LEVEL_SCENE_FILE_FILTERS } from "@/core/path/file-filters";
import { EToolId } from "@/core/routing/tool";
import { PickerForm } from "@/core/shell/chrome/PickerForm";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

/** The paths of a list row, trimmed, empty ones left out. */
function toPaths(values: ReadonlyArray<string>): Array<string> {
  return values.map((value: string) => value.trim()).filter(Boolean);
}

interface ILevelEditorOpenFormProps extends BaseComponentProps {
  /** Called once a scene has been asked for, whether it opened or failed. */
  onFinished?: () => void;
}

/**
 * The way into the editor: a scene's `.level`, where its library objects are looked for, and the game data its
 * textures are read from, with how that game data's configs read.
 */
export function LevelEditorOpenForm({
  "data-testid": dataTestId = "level-editor-open-form",
  id,
  className,
  onFinished,
}: ILevelEditorOpenFormProps): ReactElement {
  const sceneService: LevelEditorSceneService = useInjection(LevelEditorSceneService);

  const [objectsRoots, setObjectsRoots] = useState<Array<string>>([]);
  const [gameRoots, setGameRoots] = useState<Array<string>>([]);
  const [isDltx, setDltx] = useState<boolean>(false);

  const isLoading: boolean = sceneService.scene.isLoading;

  const scene: IPathField = usePathField({
    tool: EToolId.LEVEL_EDITOR,
    id: "scene",
    title: "Select scene file",
    filters: LEVEL_SCENE_FILE_FILTERS,
    isDisabled: isLoading,
  });

  const path: Nullable<string> = scene.value;

  // Remembered per root as the level viewer's is: the first game root named, which the scene's configs are read from.
  const engine: IEngineChoice = useEngineChoice(toPaths(gameRoots)[0] ?? null);

  const onOpen = useCallback(async () => {
    if (!path) {
      return;
    }

    const objects: Array<string> = toPaths(objectsRoots);
    const game: Array<string> = toPaths(gameRoots);

    await sceneService.open({
      engine: engine.choice,
      gameRoots: game.length ? createRoots(game) : null,
      isDltx,
      objectsRoots: objects.length ? objects : null,
      source: { path },
    });

    onFinished?.();
  }, [engine.choice, gameRoots, isDltx, objectsRoots, onFinished, path, sceneService]);

  return (
    <PickerForm
      data-testid={dataTestId}
      id={id}
      className={className}
      isLoading={isLoading}
      title={"Open scene"}
      description={"Reads a Level Editor scene and its part files to list, inspect and draw them. Nothing is written."}
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

      <StringListFormRow
        label={"Game roots"}
        description={
          "Installation or gamedata directories holding its textures and shaders, in order; none for gamedata beside " +
          "the scene's rawdata"
        }
        values={gameRoots}
        addLabel={"Add game root"}
        emptyLabel={"None: gamedata beside the scene's rawdata"}
        placeholder={"C:\\SDK\\editors\\gamedata"}
        isDisabled={isLoading}
        onChange={setGameRoots}
      />

      <ConfigsDialectFormRow isDltx={isDltx} isDisabled={isLoading} onChange={setDltx} />

      <EngineChoiceFormRow engine={engine} isDisabled={isLoading} />
    </PickerForm>
  );
}
