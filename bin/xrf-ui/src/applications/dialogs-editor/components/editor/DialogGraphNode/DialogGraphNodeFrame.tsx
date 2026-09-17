import { Box, Chip, Stack, Typography } from "@mui/material";
import { Handle, Position } from "@xyflow/react";
import { ReactElement } from "react";

import { IDialogGraphNodeData } from "@/applications/dialogs-editor/lib";
import { GRAPH_LAYOUT_DEFAULTS } from "@/core/graph/lib";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IDialogGraphNodeFrameProps extends BaseComponentProps {
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
  "data-testid": dataTestId = "dialog-graph-node-frame",
  id,
  className,
  data,
  isSelected,
  accent,
  hasSource,
  hasTarget,
}: IDialogGraphNodeFrameProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
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

      <Stack className={"items-center justify-between"} direction={"row"} spacing={0.5}>
        <Typography className={"text-text-secondary"} variant={"caption"} noWrap sx={{ fontFamily: "monospace" }}>
          {data.name}
        </Typography>

        {data.isFinal ? <Chip size={"small"} variant={"outlined"} label={"final"} /> : null}
      </Stack>

      <Typography
        variant={"body2"}
        title={data.label}
        className={cn(
          "line-clamp-3 wrap-anywhere",
          data.isUnresolved ? "text-text-secondary italic" : "text-text-primary not-italic"
        )}
      >
        {data.label}
      </Typography>

      {data.badges.length ? (
        <Stack direction={"row"} spacing={0.25} className={"mt-1 flex-wrap gap-y-0.5"}>
          {data.badges.map((badge: string, index: number) => (
            <Chip
              key={`${badge}-${index}`}
              className={"h-4.5 text-badge"}
              size={"small"}
              variant={"outlined"}
              label={badge}
            />
          ))}
        </Stack>
      ) : null}

      {hasSource ? <Handle type={"source"} position={Position.Bottom} isConnectable={false} /> : null}
    </Box>
  );
}
