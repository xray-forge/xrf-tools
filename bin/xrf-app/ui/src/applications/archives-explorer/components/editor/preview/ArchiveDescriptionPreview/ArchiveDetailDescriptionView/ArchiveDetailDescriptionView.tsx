import { ArchiveDescribeScope, ArchiveDetailModel } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

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
