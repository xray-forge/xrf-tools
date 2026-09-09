import { default as AddIcon } from "@mui/icons-material/Add";
import { Box, Button, Stack, Typography } from "@mui/material";
import { ReactElement } from "react";

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
    <Stack data-testid={dataTestId} id={id} className={className} spacing={1}>
      {children.length ? (
        children
      ) : (
        <Typography variant={"body2"} color={"text.secondary"}>
          {emptyLabel}
        </Typography>
      )}

      <Box>
        <Button size={"small"} disabled={isDisabled} startIcon={<AddIcon />} onClick={onAdd}>
          {addLabel}
        </Button>
      </Box>
    </Stack>
  );
}
