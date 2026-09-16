import { Typography } from "@mui/material";
import { ReactElement, useMemo, useState } from "react";

import { ArchiveDescribeScope, ArchiveShadersBlender, ArchiveShadersDescription } from "@/core/ipc/types/xrf-app";
import { EditorFilterInput } from "@/core/shell/editor/EditorFilterInput";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { filterByName } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { ArchiveShadersBlenderRow } from "./ArchiveShadersBlenderRow";

interface IArchiveShadersDescriptionViewProps extends BaseComponentProps {
  description: ArchiveShadersDescription;
  scope: ArchiveDescribeScope;
}

/**
 * The blender library: what a shader name a mesh or a level declares resolves to.
 */
export function ArchiveShadersDescriptionView({
  "data-testid": dataTestId = "archive-shaders-description-view",
  id,
  className,
  description,
  scope,
}: IArchiveShadersDescriptionViewProps): ReactElement {
  const [filter, setFilter] = useState<string>("");

  const { library, blenders } = description;

  const matched: Array<ArchiveShadersBlender> = useMemo(
    () => filterByName(blenders, filter, (blender: ArchiveShadersBlender) => blender.name),
    [blenders, filter]
  );

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Library"} isFirst>
        <ArchiveDescriptionRow
          label={"Blenders"}
          value={`${library.blenders}`}
          caption={`Across ${library.classes} ${library.classes === 1 ? "class" : "classes"}, each deciding which passes are compiled`}
        />

        <ArchiveDescriptionRow
          label={"Textures"}
          value={`${library.textures}`}
          caption={
            library.absentTextures
              ? `${library.absentTextures} not held by what is open; slots the renderer fills are not counted`
              : "Bound by name; slots the renderer fills are not counted"
          }
        />

        <ArchiveDescriptionRow
          label={"Read"}
          value={"Blender chunk only"}
          caption={"The shader script list, the constant table and the matrix table name no surface and are skipped"}
        />
      </EditorPanelSection>

      <EditorPanelSection
        title={filter.trim() ? `Blenders (${matched.length} of ${blenders.length})` : `Blenders (${blenders.length})`}
        caption={"In name order, which is how a shader name is looked up"}
      >
        <EditorFilterInput
          className={"mb-2"}
          ariaLabel={"Filter blenders"}
          query={filter}
          placeholder={"Filter blenders"}
          onQueryChange={setFilter}
        />

        {matched.length ? (
          matched.map((blender: ArchiveShadersBlender) => (
            <ArchiveShadersBlenderRow key={blender.name} blender={blender} scope={scope} />
          ))
        ) : (
          <Typography className={"text-text-disabled"} variant={"body2"}>
            No blender of this library is named that.
          </Typography>
        )}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
