import { ArchiveDescribeScope, ArchiveReference } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { describeReferenceStatus } from "./ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionReferenceLink } from "./ArchiveDescriptionReferenceLink";
import { ArchiveDescriptionRow } from "./ArchiveDescriptionRow";

interface IArchiveDescriptionReferenceProps extends BaseComponentProps {
  label: string;
  reference: ArchiveReference;
  scope: ArchiveDescribeScope;
}

/**
 * A file a description names, on a labelled line of its own.
 */
export function ArchiveDescriptionReference({
  "data-testid": dataTestId = "archive-description-reference",
  id,
  className,
  label,
  reference,
  scope,
}: IArchiveDescriptionReferenceProps): ReactElement {
  return (
    <ArchiveDescriptionRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      isMonospace
      value={<ArchiveDescriptionReferenceLink reference={reference} />}
      caption={describeReferenceStatus(reference, scope)}
    />
  );
}
