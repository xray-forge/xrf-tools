import { default as DeleteIcon } from "@mui/icons-material/Delete";
import { Stack } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
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
    <Stack
      data-testid={dataTestId}
      id={id}
      className={className}
      direction={"row"}
      spacing={1}
      sx={{ alignItems: "center" }}
    >
      {children}

      <EditorIconAction
        label={removeLabel}
        description={isDisabled ? "Wait for the current operation to finish before editing" : removeLabel}
        icon={<DeleteIcon />}
        isDisabled={isDisabled}
        onClick={onRemove}
      />
    </Stack>
  );
}
