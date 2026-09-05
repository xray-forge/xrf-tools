import { ReactElement } from "react";

import { VisualBounds } from "@/core/bindings/types/xrf-visual";
import { EditorPanelRow, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { formatCoordinate, formatVector } from "@/core/visuals/lib/visual-format";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IVisualBoundsSectionProps extends BaseComponentProps {
  title: string;
  /** Which of the two extents this is, since the rows themselves are identical. */
  caption: string;
  bounds: Nullable<VisualBounds>;
}

/** One extent, as a box and the sphere around it. */
export function VisualBoundsSection({
  "data-testid": dataTestId = "visual-bounds-section",
  id,
  className,
  title,
  caption,
  bounds,
}: IVisualBoundsSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={title} caption={caption}>
      <EditorPanelRow label={"Min"} value={formatVector(bounds?.boundingBox.min ?? null)} />
      <EditorPanelRow label={"Max"} value={formatVector(bounds?.boundingBox.max ?? null)} />
      <EditorPanelRow label={"Centre"} value={formatVector(bounds?.boundingSphere.center ?? null)} />
      <EditorPanelRow label={"Radius"} value={formatCoordinate(bounds?.boundingSphere.radius ?? null)} />
    </EditorPanelSection>
  );
}
