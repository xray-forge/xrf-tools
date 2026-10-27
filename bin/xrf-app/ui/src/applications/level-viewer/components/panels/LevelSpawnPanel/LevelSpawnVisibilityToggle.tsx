import { EditorViewToggle } from "@xrf/ui/editor/view-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { VisibilityIcon, VisibilityOffIcon } from "@xrf/ui/icons";
import { ReactElement } from "react";

import { ILevelSpawnCategoryEntry } from "@/core/level/lib/spawn/level-spawn-categories";

interface ILevelSpawnVisibilityToggleProps extends BaseComponentProps {
  entry: ILevelSpawnCategoryEntry;
  isShown: boolean;
  onToggle: () => void;
}

/**
 * The eye on a category's row, which switches the same view option as the toolbar's `Spawn` group.
 */
export function LevelSpawnVisibilityToggle({
  "data-testid": dataTestId = "level-spawn-visibility-toggle",
  id,
  className,
  entry,
  isShown,
  onToggle,
}: ILevelSpawnVisibilityToggleProps): ReactElement {
  return (
    <EditorViewToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={`Show ${entry.label.toLowerCase()}`}
      description={isShown ? `${entry.label} drawn` : `${entry.label} hidden`}
      icon={isShown ? <VisibilityIcon /> : <VisibilityOffIcon />}
      isOn={isShown}
      onToggle={onToggle}
    />
  );
}
