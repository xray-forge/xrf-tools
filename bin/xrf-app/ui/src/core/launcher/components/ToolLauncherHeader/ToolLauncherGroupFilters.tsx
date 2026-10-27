import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Chip } from "@xrf/ui/material/chip";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { ICatalogGroupFilter } from "@/core/launcher/lib";
import { EToolGroupId } from "@/core/routing/tool";

interface IToolLauncherGroupFiltersProps extends BaseComponentProps {
  filters: ReadonlyArray<ICatalogGroupFilter>;
  /** `null` is every group rather than none. */
  selectedGroupId: Nullable<EToolGroupId>;
  /** The whole roster, which the `All` chip counts even where one group is chosen. */
  totalCount: number;
  onSelectGroup: (groupId: Nullable<EToolGroupId>) => void;
}

/**
 * The chip row that narrows the catalog to one group.
 */
export function ToolLauncherGroupFilters({
  "data-testid": dataTestId = "tool-launcher-group-filters",
  id,
  className,
  filters,
  selectedGroupId,
  totalCount,
  onSelectGroup,
}: IToolLauncherGroupFiltersProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-wrap gap-1.5", className)}>
      <Chip
        aria-pressed={selectedGroupId === null}
        size={"small"}
        label={`All ${totalCount}`}
        color={selectedGroupId === null ? "primary" : "default"}
        variant={selectedGroupId === null ? "filled" : "outlined"}
        onClick={() => onSelectGroup(null)}
      />

      {filters.map(({ group, count }: ICatalogGroupFilter) => {
        const isSelected: boolean = selectedGroupId === group.id;

        return (
          <Chip
            aria-pressed={isSelected}
            key={group.id}
            size={"small"}
            label={`${group.label} ${count}`}
            color={isSelected ? "primary" : "default"}
            variant={isSelected ? "filled" : "outlined"}
            onClick={() => onSelectGroup(isSelected ? null : group.id)}
          />
        );
      })}
    </div>
  );
}
