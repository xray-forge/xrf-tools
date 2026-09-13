import { ReactElement } from "react";

import { XrayPathCollision } from "@/core/ipc/types/xrf-vfs";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveCollisionRowProps extends BaseComponentProps {
  collision: XrayPathCollision;
  isFirst?: boolean;
}

/**
 * One engine path and the two files claiming it.
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
