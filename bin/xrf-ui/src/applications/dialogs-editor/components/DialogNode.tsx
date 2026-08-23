import { Chip, Paper, Stack, Typography } from "@mui/material";
import { Handle, NodeProps, Position } from "@xyflow/react";
import { ReactElement } from "react";

import { IDialogNodeData } from "@/applications/dialogs-editor/types";
import { TGraphNode } from "@/core/graph/lib/graph.types";

/**
 * The root of one dialog: its identity and the conditions gating the whole conversation.
 */
export function DialogNode({ data, isConnectable }: NodeProps<TGraphNode<IDialogNodeData>>): ReactElement {
  return (
    <Paper variant={"outlined"} sx={{ padding: 1.5, minWidth: 200, maxWidth: 260 }}>
      <Typography variant={"subtitle2"} gutterBottom>
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
