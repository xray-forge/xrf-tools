import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { XrayPathCollision } from "@/core/ipc/types/xrf-vfs";
import { EditorPanel, EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveCollisionRow } from "./ArchiveCollisionRow";

/**
 * Every entry the open volume set holds that no engine lookup can reach.
 */
export function ArchiveCollisionsPanel({
  "data-testid": dataTestId = "archive-collisions-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  const collisions: Array<XrayPathCollision> = archivesService.collisions.value ?? [];

  if (archivesService.collisions.error || !collisions.length) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Unreachable files"}>
        <EditorPanelEmpty
          label={
            archivesService.collisions.error
              ? "Could not read what this volume set cannot reach."
              : "Every entry in this volume set resolves to a path of its own. No collisions."
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
