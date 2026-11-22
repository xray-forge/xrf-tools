import { Nullable } from "@xrf/types";
import { Button } from "@xrf/ui/button";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EditLocationAltIcon } from "@xrf/ui/icons/edit-location-alt";
import { ReactElement, useCallback } from "react";

import { ELevelEditorField } from "@/core/level/lib/editor/level-editor-field";
import { EToolId } from "@/core/routing/tool";
import { THandOverPaths, useHandOverPaths } from "@/core/ui/form/use-hand-over-paths";
import { getEditorHandOverPaths, IEditorHandOverPaths } from "@/tools/level-decompiler/lib/editor-hand-over-paths";

interface ILevelDecompileEditorActionProps extends BaseComponentProps {
  /** The scene the decompile wrote; none disables the step. */
  scene: Nullable<string>;
  /** The objects root the decompile wrote the level's own objects into; null for `objects` beside the scene. */
  objects: Nullable<string>;
  /** The libraries the decompile placed objects from. */
  libraries: ReadonlyArray<string>;
}

/**
 * Opens the level editor over a decompiled scene, with the paths {@link getEditorHandOverPaths} lays out. Its game roots
 * stay as the editor remembered them: the decompile reads none.
 */
export function LevelDecompileEditorAction({
  "data-testid": dataTestId = "level-decompile-editor-action",
  id,
  className,
  scene,
  objects,
  libraries,
}: ILevelDecompileEditorActionProps): ReactElement {
  const handOverPaths: THandOverPaths = useHandOverPaths();

  const onOpen = useCallback(() => {
    if (!scene) {
      return;
    }

    const paths: IEditorHandOverPaths = getEditorHandOverPaths(scene, objects, libraries);

    handOverPaths(EToolId.LEVEL_EDITOR, {
      [ELevelEditorField.SCENE]: paths.scene,
      [ELevelEditorField.OBJECTS]: paths.objects,
    });
  }, [handOverPaths, libraries, objects, scene]);

  return (
    <Button
      data-testid={dataTestId}
      id={id}
      className={className}
      size={"small"}
      variant={"text"}
      disabled={!scene}
      startIcon={<EditLocationAltIcon fontSize={"small"} />}
      onClick={onOpen}
    >
      Open in editor
    </Button>
  );
}
