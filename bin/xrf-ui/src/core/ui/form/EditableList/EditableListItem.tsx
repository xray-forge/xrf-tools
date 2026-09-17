import { default as DeleteIcon } from "@mui/icons-material/Delete";
import { ReactElement, ReactNode } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
