import { Box } from "@mui/material";
import { ReactElement } from "react";

import { describeTextureBadge, ETextureBadge, ITextureBadgeDescriptor } from "@/core/textures/lib/texture-catalog";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** Severity order, so a row with several badges leads with the one worth acting on. */
const BADGE_ORDER: ReadonlyArray<ETextureBadge> = [
  ETextureBadge.UNREADABLE,
  ETextureBadge.DEGRADED,
  ETextureBadge.ENGINE_SKIPPED,
  ETextureBadge.UNREFERENCED,
  ETextureBadge.ORPHAN,
  ETextureBadge.BUMPED,
  ETextureBadge.DETAIL,
];

/** Where a badge's colour comes from, since a dot has no MUI `color` prop to hand it to. */
const BADGE_PALETTE: Record<ITextureBadgeDescriptor["color"], string> = {
  default: "text.disabled",
  error: "error.main",
  info: "info.main",
  success: "success.main",
  warning: "warning.main",
};

interface ITextureBadgeMarksProps extends BaseComponentProps {
  badges: ReadonlySet<ETextureBadge>;
}

/**
 * What a texture is, as dots beside its name in the tree.
 *
 * Dots rather than chips because a tree row is a line of text and three chips make it two, and rows are what a person
 * scans a root set with. The filter chips above carry the words, and the panels carry the story; this only has to make
 * a row worth stopping on visible without being read.
 */
export function TextureBadgeMarks({ badges }: ITextureBadgeMarksProps): ReactElement | null {
  const shown: Array<ETextureBadge> = BADGE_ORDER.filter((badge: ETextureBadge) => badges.has(badge));

  if (!shown.length) {
    return null;
  }

  return (
    <Box component={"span"} sx={{ alignItems: "center", display: "inline-flex", flexShrink: 0, gap: 0.5 }}>
      {shown.map((badge: ETextureBadge) => {
        const descriptor: ITextureBadgeDescriptor = describeTextureBadge(badge);

        return (
          <Box
            key={badge}
            component={"span"}
            aria-label={descriptor.label}
            title={`${descriptor.label}: ${descriptor.description}`}
            sx={{
              backgroundColor: BADGE_PALETTE[descriptor.color],
              borderRadius: "50%",
              display: "inline-block",
              height: 7,
              width: 7,
            }}
          />
        );
      })}
    </Box>
  );
}
