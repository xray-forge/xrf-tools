import { Box, Chip, Tooltip } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { describeTextureBadge, ETextureBadge, ITextureBadgeDescriptor } from "@/core/textures/lib/texture-catalog";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ITextureBadgeFiltersProps extends BaseComponentProps {
  /** How many textures each badge would select, over the whole listing rather than the filtered view. */
  counts: ReadonlyMap<ETextureBadge, number>;
  selected: ReadonlySet<ETextureBadge>;
  onChange: (selected: ReadonlySet<ETextureBadge>) => void;
}

/**
 * Narrows the tree to the kinds of texture worth looking at.
 */
export function TextureBadgeFilters({ counts, selected, onChange }: ITextureBadgeFiltersProps): ReactElement | null {
  const onToggle = useCallback(
    (badge: ETextureBadge) => {
      const next: Set<ETextureBadge> = new Set(selected);

      if (!next.delete(badge)) {
        next.add(badge);
      }

      onChange(next);
    },
    [onChange, selected]
  );

  const shown: Array<ETextureBadge> = Object.values(ETextureBadge).filter(
    (badge: ETextureBadge) => (counts.get(badge) ?? 0) > 0 || selected.has(badge)
  );

  if (!shown.length) {
    return null;
  }

  return (
    <Box
      sx={{
        borderBottom: 1,
        borderColor: "divider",
        display: "flex",
        flexWrap: "wrap",
        gap: 0.5,
        paddingX: 1.5,
        paddingY: 1,
      }}
    >
      {shown.map((badge: ETextureBadge) => {
        const descriptor: ITextureBadgeDescriptor = describeTextureBadge(badge);
        const isSelected: boolean = selected.has(badge);

        return (
          <Tooltip key={badge} title={descriptor.description}>
            <Chip
              size={"small"}
              color={isSelected ? descriptor.color : "default"}
              variant={isSelected ? "filled" : "outlined"}
              label={`${descriptor.label} ${counts.get(badge) ?? 0}`}
              aria-pressed={isSelected}
              onClick={() => onToggle(badge)}
            />
          </Tooltip>
        );
      })}
    </Box>
  );
}
