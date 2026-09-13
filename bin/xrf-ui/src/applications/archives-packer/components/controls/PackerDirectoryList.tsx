import { FormControlLabel, Switch, TextField, Typography } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import { withDirectoryAt } from "@/applications/archives-packer/lib/pack-config";
import { ArchivePackDirectory } from "@/core/ipc/types/xrf-pack";
import { EditableList, EditableListItem } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { withoutAt } from "@/lib/types/array";

interface IPackerDirectoryListProps extends BaseComponentProps {
  directories: Array<ArchivePackDirectory>;
  isDisabled?: boolean;
  addLabel: string;
  emptyLabel: string;
  /** What the per-row switch means, which differs between including and excluding. */
  recursiveLabel: string;
  onChange: (directories: Array<ArchivePackDirectory>) => void;
}

/**
 * Editable list of directory rules, each a path relative to the packed root plus its recursive flag.
 */
export function PackerDirectoryList({
  "data-testid": dataTestId = "packer-directory-list",
  id,
  className,
  directories,
  isDisabled,
  addLabel,
  emptyLabel,
  recursiveLabel,
  onChange,
}: IPackerDirectoryListProps): ReactElement {
  return (
    <EditableList
      data-testid={dataTestId}
      id={id}
      className={className}
      addLabel={addLabel}
      emptyLabel={emptyLabel}
      isDisabled={isDisabled}
      onAdd={() => onChange([...directories, { path: "", isRecursive: true }])}
    >
      {directories.map((directory, index) => (
        <EditableListItem
          key={index}
          removeLabel={`Remove ${directory.path || "the root"}`}
          isDisabled={isDisabled}
          onRemove={() => onChange(withoutAt(directories, index))}
        >
          <TextField
            size={"small"}
            fullWidth
            disabled={isDisabled}
            value={directory.path}
            // An empty path is the packed root itself rather than a missing value.
            placeholder={"root of the source directory"}
            slotProps={{ htmlInput: { "aria-label": `Directory ${index + 1}` } }}
            onChange={(event: ChangeEvent<HTMLInputElement>) =>
              onChange(withDirectoryAt(directories, index, { path: event.target.value }))
            }
          />

          <FormControlLabel
            sx={{ flexShrink: 0, mr: 0 }}
            control={
              <Switch
                size={"small"}
                disabled={isDisabled}
                checked={directory.isRecursive}
                slotProps={{ input: { "aria-label": `${recursiveLabel} for ${directory.path || "the root"}` } }}
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  onChange(withDirectoryAt(directories, index, { isRecursive: event.target.checked }))
                }
              />
            }
            label={<Typography variant={"body2"}>{recursiveLabel}</Typography>}
          />
        </EditableListItem>
      ))}
    </EditableList>
  );
}
