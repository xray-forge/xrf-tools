import { default as AddIcon } from "@mui/icons-material/Add";
import { Box, IconButton, TextField, Tooltip, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ChangeEvent, ReactElement, useCallback, useMemo, useState } from "react";

import { VisualSequenceService } from "@/applications/visuals-sequencer/services/sequence";
import { SequencerService } from "@/applications/visuals-sequencer/services/sequencer";
import { VisualPanel, VisualPanelEmpty } from "@/core/visuals/components/panels";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/** How many matches are drawn at once. */
const SHOWN_LIMIT: number = 200;

/**
 * Every motion the open visual can play, filtered by name, each one addable to the track.
 */
export function SequenceMotionsPanel({
  "data-testid": dataTestId = "sequence-motions-panel",
  id,
  className,
}: BaseComponentProps = {}): ReactElement {
  const sequencerService: SequencerService = useInjection(SequencerService);
  const sequenceService: VisualSequenceService = useInjection(VisualSequenceService);

  const [filter, setFilter] = useState<string>("");

  const listed: Nullable<Array<string>> = sequencerService.motions.value;
  const names: Array<string> = listed ?? [];

  const matched: Array<string> = useMemo(() => {
    const needle: string = filter.trim().toLowerCase();

    return needle ? (listed ?? []).filter((name: string) => name.toLowerCase().includes(needle)) : (listed ?? []);
  }, [filter, listed]);

  /** How many clips already name each motion, so a track built by clicking the same row twice says so. */
  const used: Map<string, number> = useMemo(() => {
    const counts: Map<string, number> = new Map();

    for (const clip of sequenceService.clips) {
      counts.set(clip.motion, (counts.get(clip.motion) ?? 0) + 1);
    }

    return counts;
  }, [sequenceService.clips]);

  const onFilter = useCallback((event: ChangeEvent<HTMLInputElement>) => setFilter(event.target.value), []);

  if (sequencerService.motions.isLoading) {
    return (
      <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Motions"}>
        <VisualPanelEmpty label={"Listing motions. Every animation file the visual references is read once."} />
      </VisualPanel>
    );
  }

  if (!names.length) {
    return (
      <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Motions"}>
        <VisualPanelEmpty
          label={
            sequencerService.motions.error?.message ??
            "No motions. This visual references no animation files, so there is nothing to sequence."
          }
        />
      </VisualPanel>
    );
  }

  return (
    <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Motions"}>
      <Box
        sx={{
          position: "sticky",
          top: 0,
          zIndex: 1,
          paddingX: 2,
          paddingY: 1.5,
          backgroundColor: "background.default",
          borderBottom: 1,
          borderColor: "divider",
        }}
      >
        <TextField
          fullWidth
          size={"small"}
          value={filter}
          placeholder={"Filter motions"}
          slotProps={{ htmlInput: { "aria-label": "Filter motions" } }}
          onChange={onFilter}
        />
      </Box>

      <Box sx={{ paddingX: 1, paddingY: 1 }}>
        {matched.slice(0, SHOWN_LIMIT).map((name: string) => (
          <Box
            key={name}
            sx={{ display: "flex", alignItems: "center", gap: 1, paddingLeft: 1, paddingY: 0.2, lineHeight: 1.6 }}
          >
            <Typography variant={"body2"} sx={{ flexGrow: 1, wordBreak: "break-all" }}>
              {name}
            </Typography>

            {used.get(name) ? (
              <Typography variant={"caption"} sx={{ color: "text.disabled", flexShrink: 0 }}>
                {`×${used.get(name)}`}
              </Typography>
            ) : null}

            <Tooltip title={"Add to the track"}>
              <IconButton size={"small"} aria-label={`Add ${name}`} onClick={() => sequenceService.add(name)}>
                <AddIcon fontSize={"small"} />
              </IconButton>
            </Tooltip>
          </Box>
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
    </VisualPanel>
  );
}
