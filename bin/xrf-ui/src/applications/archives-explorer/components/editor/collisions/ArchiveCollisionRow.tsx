import { ReactElement } from "react";

import { XrayPathCollision } from "@/core/bindings/types/xrf-vfs";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveCollisionRowProps extends BaseComponentProps {
  collision: XrayPathCollision;
  isFirst?: boolean;
}

/**
 * One engine path and the two files claiming it.
 *
 * Both sites are shown as authored, because the authored spelling is exactly what the fold destroys and the only thing
 * that says which of the two to remove.
 */
export function ArchiveCollisionRow({
  "data-testid": dataTestId,
  id,
  className,
  collision,
  isFirst,
}: IArchiveCollisionRowProps): ReactElement {
  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={collision.logicalPath}
      isFirst={isFirst}
    >
      <EditorPanelProperty label={"Unreachable"} value={collision.unreachable} isMonospace />
      <EditorPanelProperty label={"Answers instead"} value={collision.kept} isMonospace />
    </EditorPanelSection>
  );
}
