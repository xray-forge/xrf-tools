import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { GridViewIcon } from "@xrf/ui/icons/grid-view";
import { TableRowsIcon } from "@xrf/ui/icons/table-rows";
import { ToggleButton, ToggleButtonGroup } from "@xrf/ui/material/toggle-button";
import { Tooltip } from "@xrf/ui/material/tooltip";
import { ReactElement } from "react";

import { TCatalogView } from "@/core/settings/lib/catalog-view";

interface IToolLauncherViewToggleProps extends BaseComponentProps {
  view: TCatalogView;
  onSelectView: (view: TCatalogView) => void;
}

/**
 * Chooses how the catalog body draws its tools; what it draws is the same either way.
 */
export function ToolLauncherViewToggle({
  "data-testid": dataTestId = "tool-launcher-view-toggle",
  id,
  className,
  view,
  onSelectView,
}: IToolLauncherViewToggleProps): ReactElement {
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
