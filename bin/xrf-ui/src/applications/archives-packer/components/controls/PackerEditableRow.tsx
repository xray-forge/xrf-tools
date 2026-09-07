import { default as DeleteIcon } from "@mui/icons-material/Delete";
import { Stack } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IPackerEditableRowProps extends BaseComponentProps {
  removeLabel: string;
  isDisabled?: boolean;
  onRemove: () => void;
  children: ReactNode;
}

/**
 * Editable fields followed by the action that removes their rule or header entry.
 */
export function PackerEditableRow({
  "data-testid": dataTestId,
  id,
  className,
  removeLabel,
  isDisabled,
  onRemove,
  children,
}: IPackerEditableRowProps): ReactElement {
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
