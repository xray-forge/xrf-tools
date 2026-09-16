import { Typography } from "@mui/material";
import { Fragment, ReactElement } from "react";

import {
  ArchiveDescribeScope,
  ArchiveLevelSurface,
  ArchiveReference,
  EArchiveLevelEntry,
  EArchiveReferenceStatus,
} from "@/core/ipc/types/xrf-app";
import { inline } from "@/lib/callbacks/inline";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { assertExhaustive } from "@/lib/types/exhaustive";

import { describeReferenceStatus } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionReferenceLink } from "../ArchiveDescriptionReferenceLink";
import { describeShaderStatus } from "./ArchiveLevelDescriptionView.utils";

interface IArchiveLevelSurfaceRowProps extends BaseComponentProps {
  surface: ArchiveLevelSurface;
  scope: ArchiveDescribeScope;
}

/**
 * One row of the shader table: the blender it draws with and the textures it binds.
 */
export function ArchiveLevelSurfaceRow({
  "data-testid": dataTestId = "archive-level-surface-row",
  id,
  className,
  surface,
  scope,
}: IArchiveLevelSurfaceRowProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("min-w-0 py-1.5 leading-panel", className)}>
      <div className={"flex min-w-0 justify-between gap-2"}>
        <Typography className={"monospace min-w-0 wrap-anywhere"} variant={"body2"}>
          {inline(() => {
            switch (surface.entry.kind) {
              case EArchiveLevelEntry.DRAWN:
                return surface.entry.shader.name;
              case EArchiveLevelEntry.UNUSABLE:
                return surface.entry.raw;
              case EArchiveLevelEntry.SKIPPED:
                return "Empty";
              default:
                return assertExhaustive(surface.entry);
            }
          })}
        </Typography>

        <Typography className={"shrink-0 whitespace-nowrap text-text-secondary"} variant={"body2"}>
          {`#${surface.index}`}
        </Typography>
      </div>

      <Typography className={"block wrap-anywhere text-text-disabled"} variant={"caption"}>
        {inline(() => {
          switch (surface.entry.kind) {
            case EArchiveLevelEntry.SKIPPED:
              return "An empty name, which the renderer skips. Every built level has one.";

            case EArchiveLevelEntry.UNUSABLE:
              return "No delimiter to split on. The renderer dereferences the result without checking it.";

            case EArchiveLevelEntry.DRAWN: {
              const shaderStatus: string | null = describeShaderStatus(surface.entry.shader, scope);
              const { textures } = surface.entry;

              return (
                <>
                  {shaderStatus ? `${shaderStatus} · ` : null}

                  {textures.length
                    ? textures.map((texture: ArchiveReference, index: number) => (
                        <Fragment key={`${index}-${texture.name}`}>
                          {index ? ", " : null}
                          <ArchiveDescriptionReferenceLink reference={texture} />
                          {texture.status === EArchiveReferenceStatus.PRESENT
                            ? null
                            : ` (${describeReferenceStatus(texture, scope)})`}
                        </Fragment>
                      ))
                    : "Binds no texture"}
                </>
              );
            }

            default:
              return assertExhaustive(surface.entry);
          }
        })}
      </Typography>
    </div>
  );
}
