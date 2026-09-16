import { ReactElement } from "react";

import { EmptyState } from "@/core/ui/layout/EmptyState";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveDescriptionPendingViewProps extends BaseComponentProps {
  /** The format's own `kind`, so the message names what was read rather than saying "this file". */
  kind: string;
}

/**
 * Scaffolding for a format the backend describes and no view draws yet.
 */
export function ArchiveDescriptionPendingView({
  "data-testid": dataTestId = "archive-description-pending-view",
  id,
  className,
  kind,
}: IArchiveDescriptionPendingViewProps): ReactElement {
  return (
    <EmptyState
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"View not written yet"}
      description={`This file was read and described as ${kind}, but no view draws that description yet.`}
    />
  );
}
