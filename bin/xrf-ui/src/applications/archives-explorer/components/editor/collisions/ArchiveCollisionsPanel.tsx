import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { XrayPathCollision } from "@/core/ipc/types/xrf-vfs";
import { EditorPanel, EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveCollisionRow } from "./ArchiveCollisionRow";

/**
 * Copies the open subject holds that no search order can rescue.
 */
export function ArchiveCollisionsPanel({
  "data-testid": dataTestId = "archive-collisions-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  const collisions: Array<XrayPathCollision> = archivesService.unreachable;

  if (archivesService.overrides.error || !collisions.length) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Unreachable files"}>
        <EditorPanelEmpty
          label={
            archivesService.overrides.error
              ? "Could not read what this subject cannot reach."
              : "Every source here reaches each of its own entries. Nothing is unreachable."
          }
        />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Unreachable files"}>
      {collisions.map((collision: XrayPathCollision, index: number) => (
        <ArchiveCollisionRow
          key={`${collision.logicalPath}:${collision.unreachable}`}
          collision={collision}
          isFirst={index === 0}
        />
      ))}
    </EditorPanel>
  );
}
