import { Chip, Stack } from "@mui/material";
import { ReactElement } from "react";

import { ArchivePackDirectory } from "@/core/ipc/types/xrf-pack";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IPackerDirectoryChipsProps extends BaseComponentProps {
  directories: Array<ArchivePackDirectory>;
  /** Said after a directory that carries its subdirectories, since the flag is what a rule turns on. */
  recursiveSuffix: string;
}

/** Directory rules as chips, each saying whether it reaches below itself. */
export function PackerDirectoryChips({
  "data-testid": dataTestId = "packer-directory-chips",
  id,
  className,
  directories,
  recursiveSuffix,
}: IPackerDirectoryChipsProps): ReactElement {
  return (
    <Stack
      data-testid={dataTestId}
      id={id}
      className={cn("flex-wrap gap-1", className)}
      direction={"row"}
      spacing={0.5}
    >
      {directories.map((directory, index) => (
        <Chip
          key={index}
          size={"small"}
          label={`${directory.path || "(root)"}${directory.isRecursive ? recursiveSuffix : ""}`}
          variant={"outlined"}
        />
      ))}
    </Stack>
  );
}
