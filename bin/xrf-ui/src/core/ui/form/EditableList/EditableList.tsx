import { default as AddIcon } from "@mui/icons-material/Add";
import { Button, Typography } from "@mui/material";
import { ReactElement } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEditableListProps extends BaseComponentProps {
  addLabel: string;
  emptyLabel: string;
  isDisabled?: boolean;
  onAdd: () => void;
  children: Array<ReactElement>;
}

/**
 * Editable rows with an empty state and an add action.
 */
export function EditableList({
  "data-testid": dataTestId,
  id,
  className,
  addLabel,
  emptyLabel,
  isDisabled,
  onAdd,
  children,
}: IEditableListProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-col gap-2", className)}>
      {children.length ? (
        children
      ) : (
        <Typography variant={"body2"} color={"text.secondary"}>
          {emptyLabel}
        </Typography>
      )}

      <div>
        <Button size={"small"} disabled={isDisabled} startIcon={<AddIcon />} onClick={onAdd}>
          {addLabel}
        </Button>
      </div>
    </div>
  );
}
