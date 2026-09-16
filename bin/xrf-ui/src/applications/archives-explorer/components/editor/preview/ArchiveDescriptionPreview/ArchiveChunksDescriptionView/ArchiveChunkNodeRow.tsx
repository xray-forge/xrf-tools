import { Typography } from "@mui/material";
import { Fragment, ReactElement } from "react";

import { ArchiveChunkNode } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
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
      <div
        data-testid={dataTestId}
        id={id}
        className={cn("flex min-w-0 justify-between gap-2 py-1.5 leading-panel", className)}
        style={{ paddingLeft: depth * 16 }}
      >
        <Typography className={"monospace min-w-0 wrap-anywhere"} variant={"body2"}>
          {formatChunkId(node.id)}

          {node.isCompressed ? (
            <Typography component={"span"} variant={"caption"} className={"ml-2 text-text-disabled"}>
              compressed
            </Typography>
          ) : null}
        </Typography>

        <Typography className={"shrink-0 whitespace-nowrap text-text-secondary"} variant={"body2"}>
          {formatBytes(node.size)}
        </Typography>
      </div>

      {node.children.map((child: ArchiveChunkNode, index: number) => (
        <ArchiveChunkNodeRow key={`${index}-${child.id}`} node={child} depth={depth + 1} />
      ))}
    </Fragment>
  );
}
