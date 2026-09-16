import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveDetailModel } from "@/core/ipc/types/xrf-app";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { ArchiveDetailModelSection } from "../ArchiveDetailModelSection";

interface IArchiveDetailDescriptionViewProps extends BaseComponentProps {
  description: ArchiveDetailModel;
  scope: ArchiveDescribeScope;
}

/**
 * A standalone detail object, which is one record and so one section.
 */
export function ArchiveDetailDescriptionView({
  "data-testid": dataTestId = "archive-detail-description-view",
  id,
  className,
  description,
  scope,
}: IArchiveDetailDescriptionViewProps): ReactElement {
  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <ArchiveDetailModelSection model={description} scope={scope} isFirst />
    </ArchiveDescriptionLayout>
  );
}
