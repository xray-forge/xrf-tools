import { EditorViewToggle } from "@xrf/ui/editor/view-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { VisibilityIcon } from "@xrf/ui/icons/visibility";
import { VisibilityOffIcon } from "@xrf/ui/icons/visibility-off";
import { ReactElement } from "react";

interface ILevelOutlinerObjectActionsProps extends BaseComponentProps {
  name: string;
  isHidden: boolean;
  onToggle: () => void;
}

/** The eye on one object's row, which hides it until revealed, as `H` hides the selection. */
export function LevelOutlinerObjectActions({
  "data-testid": dataTestId = "level-outliner-object-actions",
  id,
  className,
  name,
  isHidden,
  onToggle,
}: ILevelOutlinerObjectActionsProps): ReactElement {
  return (
    <EditorViewToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={`Hide ${name}`}
      description={isHidden ? `${name} hidden` : `${name} shown`}
      icon={isHidden ? <VisibilityOffIcon /> : <VisibilityIcon />}
      isOn={!isHidden}
      onToggle={onToggle}
    />
  );
}
