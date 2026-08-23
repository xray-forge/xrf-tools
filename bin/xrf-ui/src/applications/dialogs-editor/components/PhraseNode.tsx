import { Chip, Paper, Stack, Typography } from "@mui/material";
import { Handle, NodeProps, Position } from "@xyflow/react";
import { ReactElement } from "react";

import { IPhraseNodeData } from "@/applications/dialogs-editor/types";
import { TGraphNode } from "@/core/graph/lib/graph.types";

/**
 * One phrase: the line as it reads, and badges for the elements it carries.
 *
 * The text stays on the node and the fields do not: a dialog runs to 96 phrases, so a node holding a
 * form is both a wall of inputs and too tall to follow. Editing belongs in the inspector panel.
 */
export function PhraseNode({ data, isConnectable }: NodeProps<TGraphNode<IPhraseNodeData>>): ReactElement {
  return (
    <Paper variant={"outlined"} sx={{ padding: 1.5, minWidth: 200, maxWidth: 260 }}>
      <Handle type={"target"} position={Position.Top} isConnectable={isConnectable} />

      <Typography variant={"body2"} gutterBottom>
        {data.label}
      </Typography>

      <Stack direction={"row"} spacing={0.5} useFlexGap sx={{ flexWrap: "wrap" }}>
        {data.tags.map((tag: string) => (
          <Chip key={tag} label={tag} size={"small"} variant={"outlined"} />
        ))}
      </Stack>

      <Handle type={"source"} position={Position.Bottom} isConnectable={isConnectable} />
    </Paper>
  );
}
