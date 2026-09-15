import { Box, Typography } from "@mui/material";
import { ReactElement, useMemo, useState } from "react";

import {
  ArchiveDescribeScope,
  ArchiveLevelDescription,
  ArchiveLevelSurface,
  EArchiveLevelEntry,
} from "@/core/ipc/types/xrf-app";
import { EditorFilterInput } from "@/core/shell/editor/EditorFilterInput";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { LAYOUT } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { filterByName } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionReference } from "../ArchiveDescriptionReference";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { ArchiveLevelSurfaceRow } from "./ArchiveLevelSurfaceRow";

interface IArchiveLevelDescriptionViewProps extends BaseComponentProps {
  description: ArchiveLevelDescription;
  scope: ArchiveDescribeScope;
}

/**
 * A compiled level bundle, read through its shader table.
 */
export function ArchiveLevelDescriptionView({
  "data-testid": dataTestId = "archive-level-description-view",
  id,
  className,
  description,
  scope,
}: IArchiveLevelDescriptionViewProps): ReactElement {
  const [filter, setFilter] = useState<string>("");

  const { bundle, surfaces } = description;

  const matched: Array<ArchiveLevelSurface> = useMemo(
    () =>
      filterByName(surfaces, filter, (surface: ArchiveLevelSurface) => {
        switch (surface.entry.kind) {
          case EArchiveLevelEntry.DRAWN:
            return `${surface.entry.shader.name} ${surface.entry.textures.map((texture) => texture.name).join(" ")}`;
          case EArchiveLevelEntry.UNUSABLE:
            return surface.entry.raw;
          default:
            return "";
        }
      }),
    [filter, surfaces]
  );

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ flexGrow: 1, minWidth: 0, minHeight: 0, overflowY: "auto" }}
    >
      <Box sx={{ maxWidth: LAYOUT.readingColumnWidth }}>
        <EditorPanelSection title={"Bundle"} isFirst>
          <ArchiveDescriptionRow
            label={"Surfaces"}
            value={`${bundle.surfaces}`}
            caption={
              bundle.hasShaderTable
                ? "Rows of the shader table, which a face names by position"
                : "No shader table; the renderer refuses a bundle built without one"
            }
          />

          <ArchiveDescriptionRow
            label={"Shaders"}
            value={`${bundle.shaders}`}
            caption={
              bundle.undefinedShaders
                ? `${bundle.undefinedShaders} not defined by the library beside it`
                : "Distinct blender names the surfaces draw with"
            }
          />

          <ArchiveDescriptionRow
            label={"Textures"}
            value={`${bundle.textures}`}
            caption={
              bundle.absentTextures ? `${bundle.absentTextures} not held by what is open` : "All held by what is open"
            }
          />

          {bundle.library ? (
            <ArchiveDescriptionReference label={"Shader library"} reference={bundle.library} scope={scope} />
          ) : (
            <ArchiveDescriptionRow
              label={"Shader library"}
              value={"Not open"}
              caption={"Shader names are carried unasked; open the tree holding shaders.xr to resolve them"}
            />
          )}

          <ArchiveDescriptionRow
            label={"Built by"}
            value={`xrLC ${bundle.xrlcVersion}`}
            caption={`Quality ${bundle.xrlcQuality}`}
          />
        </EditorPanelSection>

        <EditorPanelSection
          title={filter.trim() ? `Surfaces (${matched.length} of ${surfaces.length})` : `Surfaces (${surfaces.length})`}
          caption={"In table order, which is the order a face addresses them in"}
        >
          <EditorFilterInput
            query={filter}
            placeholder={"Filter by shader or texture"}
            ariaLabel={"Filter by shader or texture"}
            onQueryChange={setFilter}
            sx={{ marginBottom: 1 }}
          />

          {matched.length ? (
            matched.map((surface: ArchiveLevelSurface) => (
              <ArchiveLevelSurfaceRow key={surface.index} surface={surface} scope={scope} />
            ))
          ) : (
            <Typography variant={"body2"} sx={{ color: "text.disabled" }}>
              No surface of this level names that.
            </Typography>
          )}
        </EditorPanelSection>
      </Box>
    </Box>
  );
}
