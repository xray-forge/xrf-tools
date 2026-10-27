import { useInjection } from "@wirestate/react";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Inventory2Icon } from "@xrf/ui/icons/inventory2";
import { ReactElement } from "react";

import { ILevelSpawnCategoryEntry, LEVEL_SPAWN_CATEGORIES } from "@/core/level/lib/spawn/level-spawn-categories";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelViewService } from "@/core/level/services/level-view";

/**
 * Which of the level's spawned objects are drawn, by what the game spawns them as.
 */
export function LevelSpawnAction({
  "data-testid": dataTestId = "level-spawn-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const options: ILevelViewOptions = viewService.options;

  const shown: Array<string> = LEVEL_SPAWN_CATEGORIES.filter(
    (entry: ILevelSpawnCategoryEntry) => options[entry.option]
  ).map((entry: ILevelSpawnCategoryEntry) => entry.label.toLowerCase());

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Spawn"}
      description={
        shown.length
          ? `Spawned ${shown.join(", ")}${options.isSpawnedReleased ? ", released too" : ""}`
          : "Nothing spawned drawn"
      }
      icon={<Inventory2Icon />}
      isActive={shown.length > 0}
    >
      {LEVEL_SPAWN_CATEGORIES.map((entry: ILevelSpawnCategoryEntry) => (
        <EditorPopoverGroupSection
          key={entry.category}
          label={entry.label}
          isOn={options[entry.option]}
          onToggle={() => viewService.toggleOption(entry.option)}
        />
      ))}

      <EditorPopoverGroupSection
        label={"Released"}
        description={"Draws what a new game releases before the player arrives, with its group"}
        isOn={options.isSpawnedReleased}
        onToggle={() => viewService.toggleOption("isSpawnedReleased")}
      />
    </EditorPopoverGroup>
  );
}
