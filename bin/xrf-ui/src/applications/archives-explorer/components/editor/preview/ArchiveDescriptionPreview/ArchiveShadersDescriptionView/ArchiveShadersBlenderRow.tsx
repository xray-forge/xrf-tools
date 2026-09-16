import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveShadersBlender, ArchiveShadersProperty } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { describeReferenceStatus } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionReferenceLink } from "../ArchiveDescriptionReferenceLink";

interface IArchiveShadersBlenderRowProps extends BaseComponentProps {
  blender: ArchiveShadersBlender;
  scope: ArchiveDescribeScope;
}

/**
 * One blender: the shader name, the class that decides its passes, and the grid an author left behind.
 */
export function ArchiveShadersBlenderRow({
  "data-testid": dataTestId = "archive-shaders-blender-row",
  id,
  className,
  blender,
  scope,
}: IArchiveShadersBlenderRowProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("min-w-0 py-1.5 leading-panel", className)}>
      <div className={"flex min-w-0 justify-between gap-2"}>
        <Typography className={"monospace min-w-0 wrap-anywhere"} variant={"body2"}>
          {blender.name}
        </Typography>

        <Typography className={"shrink-0 whitespace-nowrap text-text-secondary"} variant={"body2"}>
          {`${blender.class} · v${blender.version}`}
        </Typography>
      </div>

      {blender.properties.map((property: ArchiveShadersProperty, index: number) => (
        <Typography
          key={`${index}-${property.name}`}
          className={"block wrap-anywhere text-text-disabled"}
          variant={"caption"}
        >
          {`${property.name}: `}

          {property.texture ? (
            <>
              <ArchiveDescriptionReferenceLink reference={property.texture} />
              {` — ${describeReferenceStatus(property.texture, scope) ?? ""}`}
            </>
          ) : (
            property.value || property.kind.toLowerCase()
          )}
        </Typography>
      ))}
    </div>
  );
}
