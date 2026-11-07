import { LtxDeclaration } from "@xrf/ipc/types/xrf-ltx-inspect";
import { getPathName } from "@xrf/path";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Chip } from "@xrf/ui/material/chip";
import { ListItem, ListItemButton } from "@xrf/ui/material/list";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement, useCallback } from "react";

import { describeDeclaration } from "@/tools/configs-explorer/lib/declaration";

interface IConfigsFindRowProps extends BaseComponentProps {
  declaration: LtxDeclaration;
  /** The row was chosen, to open its config at the statement's line. */
  onOpen: (declaration: LtxDeclaration) => void;
}

/**
 * One found statement: whether it declares or patches, where it is written, and the line as the file spells it.
 */
export function ConfigsFindRow({
  "data-testid": dataTestId = "configs-find-row",
  declaration,
  onOpen,
}: IConfigsFindRowProps): ReactElement {
  const onClick = useCallback(() => onOpen(declaration), [declaration, onOpen]);

  return (
    <ListItem data-testid={dataTestId} className={"block"} divider disablePadding>
      <ListItemButton className={"block"} onClick={onClick}>
        <div className={"flex min-w-0 flex-wrap items-center gap-1.5"}>
          <Chip size={"small"} variant={"outlined"} label={declaration.isPatch ? "patch" : "declaration"} />

          <Typography
            className={"monospace min-w-0 wrap-anywhere text-text-secondary"}
            component={"span"}
            variant={"caption"}
            title={`${declaration.path} (${declaration.source})`}
          >
            {`${getPathName(declaration.path) || declaration.path}:${declaration.line}`}
          </Typography>

          {/* Said only where it is true: an archived config reads like a loose one and only cannot be written. */}
          {declaration.isPhysical ? null : (
            <Typography className={"text-text-secondary"} component={"span"} variant={"caption"}>
              archived
            </Typography>
          )}
        </div>

        <Typography className={"monospace mt-1 wrap-anywhere"} variant={"body2"}>
          {describeDeclaration(declaration)}
        </Typography>
      </ListItemButton>
    </ListItem>
  );
}
