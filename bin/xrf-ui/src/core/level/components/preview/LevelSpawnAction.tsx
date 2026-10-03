import { default as Inventory2Icon } from "@mui/icons-material/Inventory2";
import { ReactElement } from "react";

import { ILevelSpawnCategoryEntry, LEVEL_SPAWN_CATEGORIES } from "@/core/level/lib/spawn/level-spawn-categories";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelSpawnActionProps extends BaseComponentProps {
  options: ILevelViewOptions;
  onToggle: (option: keyof ILevelViewOptions) => void;
}

/**
 * Which of the level's spawned objects are drawn, by what the game spawns them as.
 */
export function LevelSpawnAction({
  "data-testid": dataTestId = "level-spawn-action",
  id,
  className,
  options,
  onToggle,
}: ILevelSpawnActionProps): ReactElement {
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
          onToggle={() => onToggle(entry.option)}
        />
      ))}

      <EditorPopoverGroupSection
        label={"Released"}
        description={"What a new game releases before the actor arrives, drawn with its group"}
        isOn={options.isSpawnedReleased}
        onToggle={() => onToggle("isSpawnedReleased")}
      />
    </EditorPopoverGroup>
  );
}
