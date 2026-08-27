import { Box, Chip, Stack, Typography } from "@mui/material";
import { Handle, Position } from "@xyflow/react";
import { ReactElement } from "react";

import { IDialogGraphNodeData } from "@/applications/dialogs-editor/lib";
import { GRAPH_LAYOUT_DEFAULTS } from "@/core/graph/lib";

/** Lines of text a node shows before clamping. Past this the node stops being scannable at a glance. */
const LABEL_LINES: number = 3;

export interface IDialogGraphNodeFrameProps {
  data: IDialogGraphNodeData;
  isSelected: boolean;
  /** Left border colour, which is how each node kind states what it is without a second label. */
  accent: string;
  /** Whether anything can lead out of this node. False only where the domain has no such edge. */
  hasSource: boolean;
  /** Whether anything can lead into it. False on the dialog root, which starts the conversation. */
  hasTarget: boolean;
}

/**
 * The frame both dialog node kinds draw in.
 *
 * Shared because the two differ only in their accent and which handles they carry — everything a
 * reader looks at is the same, so drawing it twice would let the two drift apart visually.
 *
 * Handles render but never connect: this canvas is read-only, and a handle that refuses reads better
 * than one that is absent, because it says where a link will attach once editing exists.
 *
 * Sized from the layouter's own default rather than a matching literal, so a node and the rank holding
 * it cannot disagree about how wide it is.
 */
export function DialogGraphNodeFrame({
  data,
  isSelected,
  accent,
  hasSource,
  hasTarget,
}: IDialogGraphNodeFrameProps): ReactElement {
  return (
    <Box
      sx={{
        backgroundColor: "background.paper",
        border: "1px solid",
        borderColor: isSelected ? "primary.main" : "divider",
        borderLeft: "3px solid",
        borderLeftColor: accent,
        ...(data.isTerminal ? { borderBottom: "2px solid", borderBottomColor: "text.disabled" } : {}),
        borderRadius: 1,
        boxShadow: isSelected ? 3 : 0,
        padding: 1,
        width: GRAPH_LAYOUT_DEFAULTS.nodeWidth,
      }}
    >
      {hasTarget ? <Handle type={"target"} position={Position.Top} isConnectable={false} /> : null}

      <Stack direction={"row"} spacing={0.5} sx={{ alignItems: "center", justifyContent: "space-between" }}>
        <Typography variant={"caption"} noWrap sx={{ color: "text.secondary", fontFamily: "monospace" }}>
          {data.name}
        </Typography>

        {data.isFinal ? <Chip size={"small"} variant={"outlined"} label={"final"} /> : null}
      </Stack>

      <Typography
        variant={"body2"}
        title={data.label}
        sx={{
          color: data.isUnresolved ? "text.secondary" : "text.primary",
          display: "-webkit-box",
          fontStyle: data.isUnresolved ? "italic" : "normal",
          overflow: "hidden",
          WebkitBoxOrient: "vertical",
          WebkitLineClamp: LABEL_LINES,
          wordBreak: "break-word",
        }}
      >
        {data.label}
      </Typography>

      {data.badges.length ? (
        <Stack direction={"row"} spacing={0.25} sx={{ flexWrap: "wrap", marginTop: 0.5, rowGap: 0.25 }}>
          {data.badges.map((badge: string, index: number) => (
            <Chip
              key={`${badge}-${index}`}
              size={"small"}
              variant={"outlined"}
              label={badge}
              sx={{ height: 18, fontSize: 10 }}
            />
          ))}
        </Stack>
      ) : null}

      {hasSource ? <Handle type={"source"} position={Position.Bottom} isConnectable={false} /> : null}
    </Box>
  );
}
