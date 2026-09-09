import { Chip, Stack } from "@mui/material";
import { ReactElement } from "react";

import { LtxInventoryFile } from "@/core/bindings/types/xrf-ltx-inspect";
import { inline } from "@/lib/callbacks/inline";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IConfigsMenuBadgesProps extends BaseComponentProps {
  file: LtxInventoryFile;
}

/**
 * What one config is to the project, beside its name in the tree.
 */
export function ConfigsMenuBadges({
  "data-testid": dataTestId = "configs-menu-badges",
  file,
}: IConfigsMenuBadgesProps): ReactElement {
  // Switched on the role's own discriminator rather than tested against it three times, so adding a role to the
  // inventory is a compiler error here instead of a row that silently loses its badge.
  const role: Nullable<string> = inline(() => {
    switch (file.role.kind) {
      case "entryPoint":
        return "entry";
      case "schemeFile":
        return "scheme";
      case "attachment":
        return "patch";
      // Almost every config in a game tree is included, and a badge on nearly every row is a badge nobody reads.
      case "included":
        return null;
    }
  });

  return (
    <Stack data-testid={dataTestId} direction={"row"} spacing={0.5} sx={{ alignItems: "center" }}>
      {role ? <Chip size={"small"} variant={"outlined"} label={role} /> : null}

      {file.isPhysical ? null : (
        <Chip
          size={"small"}
          variant={"outlined"}
          color={"warning"}
          label={"archived"}
          title={"Read from an archive volume; nothing can write to it in place"}
        />
      )}
    </Stack>
  );
}
