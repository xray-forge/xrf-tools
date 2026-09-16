import { ReactElement } from "react";

import { XrayPathCollision } from "@/core/ipc/types/xrf-vfs";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveUnreachableRow } from "./ArchiveUnreachableRow";

export interface IArchiveUnreachableSectionProps extends BaseComponentProps {
  collisions: ReadonlyArray<XrayPathCollision>;
}

/**
 * Copies of the open subject that no search order can rescue.
 */
export function ArchiveUnreachableSection({
  "data-testid": dataTestId = "archive-unreachable-section",
  id,
  className,
  collisions,
}: IArchiveUnreachableSectionProps): ReactElement {
  return (
    <DetailSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Unreachable copies"}
      description={
        collisions.length
          ? "Two entries of one source fold to one engine identity, and inside a single source there is no search " +
            "order to appeal to - so one of them can never be reached. That makes it an authoring fault in the " +
            "archive rather than layering: an override is a different source winning as intended. Both spellings " +
            "are given as authored, because case folding is what destroys them and which one to remove depends on " +
            "knowing them."
          : "Every source here reaches each of its own entries. No two entries of one source fold to the same " +
            "engine identity, so nothing is hidden by an authoring fault."
      }
      fact={`${collisions.length} cop${collisions.length === 1 ? "y" : "ies"}`}
    >
      {collisions.length ? (
        <div className={"max-h-40 overflow-y-auto"}>
          {collisions.map((collision: XrayPathCollision) => (
            <ArchiveUnreachableRow key={`${collision.logicalPath}:${collision.unreachable}`} collision={collision} />
          ))}
        </div>
      ) : null}
    </DetailSection>
  );
}
