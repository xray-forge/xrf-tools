import { formatCount } from "@xrf/format";
import { XrayPathCollision } from "@xrf/ipc/types/xrf-vfs";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

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
          ? "Two entries of one source fold to one engine identity, so one can never be reached - an authoring " +
            "fault in the archive rather than layering."
          : "Every source here reaches each of its own entries, so nothing is hidden by an authoring fault."
      }
      fact={`${formatCount(collisions.length)} cop${collisions.length === 1 ? "y" : "ies"}`}
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
