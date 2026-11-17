import { useInjection } from "@wirestate/react";
import { SCENE_HELPER_TOOLS, SceneHelperToolCount, SceneHelperToolEntry } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { VisibilityIcon } from "@xrf/ui/icons/visibility";
import { ReactElement } from "react";

import { LevelEditorHelperToolRow } from "@/tools/level-editor/components/toolbar/LevelEditorHelperToolRow";
import {
  ILevelEditorHelperOptions,
  isLevelEditorHelperOn,
  LEVEL_EDITOR_HELPERS_OFF_REASON,
} from "@/tools/level-editor/lib/helpers/level-editor-helper-options";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

/**
 * What the editor shows over the scene beside its objects, as an editor's show menu gathers it: the helpers' master
 * switch heading a switch for each helped tool, with the picture its helpers draw and how many the open scene holds,
 * the spawn points' switch, and the master one over it, switching the spawn points' models too; and the switches of the
 * surfaces the game never draws and of those dressed in the SDK's marker textures, each off until asked for.
 */
export function LevelEditorShowAction({
  "data-testid": dataTestId = "level-editor-show-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const helpersService: LevelEditorHelpersService = useInjection(LevelEditorHelpersService);
  const sceneService: LevelEditorSceneService = useInjection(LevelEditorSceneService);
  const helpers: ILevelEditorHelperOptions = helpersService.helpers;
  const counts: ReadonlyArray<SceneHelperToolCount> = sceneService.helpers.value?.tools ?? [];
  const isOff: boolean = !helpers.isShown;

  const shown: number = SCENE_HELPER_TOOLS.filter((entry: SceneHelperToolEntry) =>
    isLevelEditorHelperOn(helpers, entry.tool)
  ).length;

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Show"}
      description={isOff ? "No helpers" : `Helpers of ${shown} of ${SCENE_HELPER_TOOLS.length} tools`}
      icon={<VisibilityIcon />}
      isActive={helpers.isShown}
    >
      <EditorPopoverGroupSection
        label={"Helpers"}
        description={
          "Draws the wires and marks of what the scene holds with no geometry: spawn points, ways and the rest. " +
          "Spawn points drawn as their models switch with them."
        }
        isOn={helpers.isShown}
        onToggle={helpersService.toggleHelpers}
      >
        {SCENE_HELPER_TOOLS.map((entry: SceneHelperToolEntry) => {
          const isOn: boolean = isLevelEditorHelperOn(helpers, entry.tool);

          return (
            <LevelEditorHelperToolRow
              key={entry.tool}
              entry={entry}
              count={counts.find((counted: SceneHelperToolCount) => counted.tool === entry.tool)?.count ?? 0}
              isOn={isOn}
              isDisabled={isOff}
              reason={LEVEL_EDITOR_HELPERS_OFF_REASON}
              onChange={() => helpersService.setHelperTools({ isOn: !isOn, tools: [entry.tool] })}
            />
          );
        })}
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={"Collision surfaces"}
        description={
          "Draws the surfaces the game never draws, which their compiler shader keeps for collision or shadows alone: " +
          "a tree's collision hull, a level's invisible walls."
        }
        isOn={helpersService.isCollisionShown}
        onToggle={helpersService.toggleCollision}
      />

      <EditorPopoverGroupSection
        label={"Editor-only textures"}
        description={
          "Draws the surfaces dressed in the SDK's marker textures, which only show the editor what a level holds " +
          "unseen: a sun blocker, an occluder's sheet."
        }
        isOn={helpersService.isEditorOnlyShown}
        onToggle={helpersService.toggleEditorOnly}
      />
    </EditorPopoverGroup>
  );
}
