import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveDetailModel } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { NOT_DECLARED } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionReference } from "../ArchiveDescriptionReference";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeModelBounds, describeModelMesh, describeModelScale } from "./ArchiveDetailModelSection.utils";

interface IArchiveDetailModelSectionProps extends BaseComponentProps {
  model: ArchiveDetailModel;
  scope: ArchiveDescribeScope;
  /** Whether this section leads the panel, which a standalone object does and a library entry does not. */
  isFirst?: boolean;
  /** Shown instead of `Detail object`, for a library naming which of its entries this is. */
  title?: string;
}

/**
 * One detail object, stated whole.
 */
export function ArchiveDetailModelSection({
  "data-testid": dataTestId = "archive-detail-model-section",
  id,
  className,
  model,
  scope,
  isFirst = false,
  title = "Detail object",
}: IArchiveDetailModelSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={title} isFirst={isFirst}>
      {model.texture ? (
        <ArchiveDescriptionReference label={"Texture"} reference={model.texture} scope={scope} />
      ) : (
        <ArchiveDescriptionRow label={"Texture"} value={NOT_DECLARED} />
      )}

      <ArchiveDescriptionRow
        label={"Shader"}
        value={model.shader || NOT_DECLARED}
        isMonospace={Boolean(model.shader)}
        caption={"A blender defined in `shaders.xr` rather than a file, so there is nothing to select"}
      />

      <ArchiveDescriptionRow
        label={"Mesh"}
        value={describeModelMesh(model)}
        caption={describeModelBounds(model.bounds) ?? "The object carries no mesh at all"}
      />

      <ArchiveDescriptionRow
        label={"Scale"}
        value={describeModelScale(model)}
        caption={"What a slot may multiply the object by when it plants one"}
      />

      <ArchiveDescriptionRow
        label={"Sways"}
        value={model.isWaving ? "Yes" : "No"}
        caption={
          model.isWaving
            ? "The vertex shader moves it with the wind"
            : "The object declares `DO_NO_WAVING`, so the wind leaves it alone"
        }
      />

      {model.unnamedFlags ? (
        <ArchiveDescriptionRow
          label={"Unnamed flags"}
          value={`0x${(model.unnamedFlags >>> 0).toString(16).toUpperCase()}`}
          caption={"Bits of the flag word the engine gives no name"}
        />
      ) : null}
    </EditorPanelSection>
  );
}
