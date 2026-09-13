import { default as GridViewIcon } from "@mui/icons-material/GridView";
import { default as TableRowsIcon } from "@mui/icons-material/TableRows";
import { ToggleButton, ToggleButtonGroup, Tooltip } from "@mui/material";
import { ReactElement } from "react";

import { TCatalogView } from "@/core/settings/lib/catalog-view";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IApplicationLauncherViewToggleProps extends BaseComponentProps {
  view: TCatalogView;
  onSelectView: (view: TCatalogView) => void;
}

/**
 * Chooses how the catalog body draws its tools; what it draws is the same either way.
 */
export function ApplicationLauncherViewToggle({
  "data-testid": dataTestId = "application-launcher-view-toggle",
  id,
  className,
  view,
  onSelectView,
}: IApplicationLauncherViewToggleProps): ReactElement {
  return (
    <ToggleButtonGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      aria-label={"Catalog view"}
      exclusive={true}
      size={"small"}
      value={view}
      // Releasing the active button would leave the catalog with no view at all.
      onChange={(_, next: Nullable<TCatalogView>) => next && onSelectView(next)}
    >
      <Tooltip title={"Row view"}>
        <ToggleButton aria-label={"Row view"} value={"rows"}>
          <TableRowsIcon fontSize={"small"} />
        </ToggleButton>
      </Tooltip>

      <Tooltip title={"Grid view"}>
        <ToggleButton aria-label={"Grid view"} value={"grid"}>
          <GridViewIcon fontSize={"small"} />
        </ToggleButton>
      </Tooltip>
    </ToggleButtonGroup>
  );
}
