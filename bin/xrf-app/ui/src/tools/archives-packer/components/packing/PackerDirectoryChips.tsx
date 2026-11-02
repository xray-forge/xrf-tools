import { ArchivePackDirectory } from "@xrf/ipc/types/xrf-pack";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Chip } from "@xrf/ui/material/chip";
import { Stack } from "@xrf/ui/material/stack";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

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
