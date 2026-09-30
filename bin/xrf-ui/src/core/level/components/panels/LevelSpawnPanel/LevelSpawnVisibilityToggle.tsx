import { default as VisibilityIcon } from "@mui/icons-material/Visibility";
import { default as VisibilityOffIcon } from "@mui/icons-material/VisibilityOff";
import { ReactElement } from "react";

import { ILevelSpawnCategoryEntry } from "@/core/level/lib/spawn/level-spawn-categories";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
    <EditorIconAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={`${isShown ? "Hide" : "Show"} ${entry.label.toLowerCase()}`}
      description={isShown ? `${entry.label} drawn` : `${entry.label} hidden`}
      icon={isShown ? <VisibilityIcon /> : <VisibilityOffIcon />}
      aria-pressed={isShown}
      onClick={onToggle}
    />
  );
}
