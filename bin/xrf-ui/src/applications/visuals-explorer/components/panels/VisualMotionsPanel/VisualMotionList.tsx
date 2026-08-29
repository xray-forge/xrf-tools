import { Box, ListItemButton, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { VisualPanelEmpty } from "@/core/visuals/components/panels";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/** How many matches are drawn at once, as the sequencer's list caps them. */
const SHOWN_LIMIT: number = 200;

export interface IVisualMotionListProps extends BaseComponentProps {
  /** What the panel's filter field holds, already the user's whole query. */
  filter: string;
}

/**
 * Every motion the open visual can play, narrowed by the panel's filter, each one posed by a click.
 *
 * A filtered list rather than the autocomplete the footer bar used: search stays how a name is found among the
 * thousands a character references, and a column has the height to show what matched, which a bar did not.
 */
export function VisualMotionList({
  "data-testid": dataTestId = "visual-motion-list",
  id,
  className,
  filter,
}: IVisualMotionListProps): ReactElement {
  const service: VisualMotionService = useInjection(VisualMotionService);

  // The loadable's own value is what the memo depends on: `names` defaults to a fresh array every render, which as a
  // dependency would refilter on each one.
  const listed: Nullable<Array<string>> = service.motions.value;
  const names: Array<string> = listed ?? [];
  const posed: Nullable<string> = service.posed.value?.bake.name ?? null;

  const matched: Array<string> = useMemo(() => {
    const needle: string = filter.trim().toLowerCase();

    return needle ? (listed ?? []).filter((name: string) => name.toLowerCase().includes(needle)) : (listed ?? []);
  }, [filter, listed]);

  if (service.motions.isLoading) {
    return <VisualPanelEmpty label={"Listing motions. Every animation file the visual references is read once."} />;
  }

  if (!names.length) {
    return (
      <VisualPanelEmpty
        label={service.motions.error?.message ?? "This visual references animation files that name no motions."}
      />
    );
  }

  return (
    <Box data-testid={dataTestId} id={id} className={className} sx={{ paddingX: 1 }}>
      {matched.slice(0, SHOWN_LIMIT).map((name: string) => (
        <ListItemButton
          key={name}
          dense
          selected={name === posed}
          sx={{ borderRadius: 1, paddingY: 0.2 }}
          onClick={() => void service.open(name)}
        >
          <Typography variant={"body2"} sx={{ wordBreak: "break-all" }}>
            {name}
          </Typography>
        </ListItemButton>
      ))}

      {matched.length === 0 ? (
        <VisualPanelEmpty label={`No motion of the ${names.length} this visual plays matches that.`} />
      ) : null}

      {matched.length > SHOWN_LIMIT ? (
        <Typography variant={"caption"} sx={{ display: "block", padding: 1, color: "text.disabled" }}>
          {`Showing ${SHOWN_LIMIT} of ${matched.length} matches. Narrow the filter to reach the rest.`}
        </Typography>
      ) : null}
    </Box>
  );
}
