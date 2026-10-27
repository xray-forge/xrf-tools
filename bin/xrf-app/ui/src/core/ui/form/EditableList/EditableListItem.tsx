import { BaseComponentProps } from "@xrf/ui/element-types";
import { DeleteIcon } from "@xrf/ui/icons";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement, ReactNode } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";

interface IEditableListItemProps extends BaseComponentProps {
  removeLabel: string;
  isDisabled?: boolean;
  onRemove: () => void;
  children: ReactNode;
}

/**
 * Editable fields with a remove action.
 */
export function EditableListItem({
  "data-testid": dataTestId,
  id,
  className,
  removeLabel,
  isDisabled,
  onRemove,
  children,
}: IEditableListItemProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-row items-center gap-2", className)}>
      {children}

      <EditorIconAction
        label={removeLabel}
        description={isDisabled ? "Wait for the current operation to finish before editing" : removeLabel}
        icon={<DeleteIcon />}
        isDisabled={isDisabled}
        onClick={onRemove}
      />
    </div>
  );
}
