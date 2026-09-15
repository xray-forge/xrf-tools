import { Box, Typography } from "@mui/material";
import { Fragment, ReactElement } from "react";

import {
  ArchiveDescribeScope,
  ArchiveLevelSurface,
  ArchiveReference,
  EArchiveLevelEntry,
  EArchiveReferenceStatus,
} from "@/core/ipc/types/xrf-app";
import { MONOSPACE, PANEL } from "@/core/theme/tokens";
import { inline } from "@/lib/callbacks/inline";
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
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ minWidth: 0, paddingY: PANEL.propertyPaddingY, lineHeight: PANEL.contentLineHeight }}
    >
      <Box sx={{ display: "flex", gap: 1, justifyContent: "space-between", minWidth: 0 }}>
        <Typography variant={"body2"} sx={{ ...MONOSPACE, minWidth: 0, overflowWrap: "anywhere" }}>
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

        <Typography variant={"body2"} sx={{ color: "text.secondary", flexShrink: 0, whiteSpace: "nowrap" }}>
          {`#${surface.index}`}
        </Typography>
      </Box>

      <Typography variant={"caption"} sx={{ display: "block", color: "text.disabled", overflowWrap: "anywhere" }}>
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
    </Box>
  );
}
