import { ArchiveDescribeScope, ArchiveDetailModel } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

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
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <ArchiveDetailModelSection model={description} scope={scope} />
    </ReadingColumn>
  );
}
