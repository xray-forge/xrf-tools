import { default as AddIcon } from "@mui/icons-material/Add";
import { Box, Button, Stack, Typography } from "@mui/material";
import { ReactElement } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

interface IPackerEditableListProps extends BaseComponentProps {
  addLabel: string;
  emptyLabel: string;
  isDisabled?: boolean;
  onAdd: () => void;
  children: Array<ReactElement>;
}

/**
 * Rule-list spacing, empty state, and the action that appends a new row.
 */
export function PackerEditableList({
  "data-testid": dataTestId,
  id,
  className,
  addLabel,
  emptyLabel,
  isDisabled,
  onAdd,
  children,
}: IPackerEditableListProps): ReactElement {
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
