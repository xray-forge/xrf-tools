import { Box, Typography } from "@mui/material";
import { Fragment, ReactElement } from "react";

import { ArchiveChunkNode } from "@/core/ipc/types/xrf-app";
import { MONOSPACE, PANEL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

import { formatChunkId } from "../ArchiveDescriptionPreview.utils";

interface IArchiveChunkNodeRowProps extends BaseComponentProps {
  node: ArchiveChunkNode;
  depth?: number;
}

/**
 * One chunk of a container, with whatever its payload turned out to hold beneath it.
 */
export function ArchiveChunkNodeRow({
  "data-testid": dataTestId = "archive-chunk-node-row",
  id,
  className,
  node,
  depth = 0,
}: IArchiveChunkNodeRowProps): ReactElement {
  return (
    <Fragment>
      <Box
        data-testid={dataTestId}
        id={id}
        className={className}
        sx={{
          display: "flex",
          gap: 1,
          justifyContent: "space-between",
          minWidth: 0,
          paddingY: PANEL.propertyPaddingY,
          paddingLeft: depth * 2,
          lineHeight: PANEL.contentLineHeight,
        }}
      >
        <Typography variant={"body2"} sx={{ ...MONOSPACE, minWidth: 0, overflowWrap: "anywhere" }}>
          {formatChunkId(node.id)}

          {node.isCompressed ? (
            <Typography component={"span"} variant={"caption"} sx={{ color: "text.disabled", marginLeft: 1 }}>
              compressed
            </Typography>
          ) : null}
        </Typography>

        <Typography variant={"body2"} sx={{ color: "text.secondary", flexShrink: 0, whiteSpace: "nowrap" }}>
          {formatBytes(node.size)}
        </Typography>
      </Box>

      {node.children.map((child: ArchiveChunkNode, index: number) => (
        <ArchiveChunkNodeRow key={`${index}-${child.id}`} node={child} depth={depth + 1} />
      ))}
    </Fragment>
  );
}
