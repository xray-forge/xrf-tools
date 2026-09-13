import { Box, Chip } from "@mui/material";
import { ReactElement } from "react";

import { ICatalogGroupFilter } from "@/core/launcher/lib";
import { EApplicationGroupId } from "@/core/routing/application";
import { getControlBackgroundSx } from "@/core/theme/control-background";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IApplicationLauncherGroupFiltersProps extends BaseComponentProps {
  filters: ReadonlyArray<ICatalogGroupFilter>;
  /** `null` is every group rather than none. */
  selectedGroupId: Nullable<EApplicationGroupId>;
  /** The whole roster, which the `All` chip counts even where one group is chosen. */
  totalCount: number;
  onSelectGroup: (groupId: Nullable<EApplicationGroupId>) => void;
}

/**
 * The chip row that narrows the catalog to one group.
 */
export function ApplicationLauncherGroupFilters({
  "data-testid": dataTestId = "application-launcher-group-filters",
  id,
  className,
  filters,
  selectedGroupId,
  totalCount,
  onSelectGroup,
}: IApplicationLauncherGroupFiltersProps): ReactElement {
  return (
    <Box data-testid={dataTestId} id={id} className={className} sx={{ display: "flex", flexWrap: "wrap", gap: 0.75 }}>
      <Chip
        aria-pressed={selectedGroupId === null}
        size={"small"}
        label={`All ${totalCount}`}
        color={selectedGroupId === null ? "primary" : "default"}
        variant={selectedGroupId === null ? "filled" : "outlined"}
        sx={selectedGroupId === null ? undefined : getControlBackgroundSx}
        onClick={() => onSelectGroup(null)}
      />

      {filters.map(({ group, count }: ICatalogGroupFilter) => {
        const isSelected: boolean = selectedGroupId === group.id;

        return (
          <Chip
            key={group.id}
            size={"small"}
            label={`${group.label} ${count}`}
            aria-pressed={isSelected}
            color={isSelected ? "primary" : "default"}
            variant={isSelected ? "filled" : "outlined"}
            sx={isSelected ? undefined : getControlBackgroundSx}
            onClick={() => onSelectGroup(isSelected ? null : group.id)}
          />
        );
      })}
    </Box>
  );
}
