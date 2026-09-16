import { ReactElement } from "react";

import { ArchiveOmfDescription } from "@/core/ipc/types/xrf-app";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { ArchiveOmfBankSection } from "./ArchiveOmfBankSection";
import { ArchiveOmfMotionsSection } from "./ArchiveOmfMotionsSection";
import { ArchiveOmfPartitionSection } from "./ArchiveOmfPartitionSection";

interface IArchiveOmfDescriptionViewProps extends BaseComponentProps {
  description: ArchiveOmfDescription;
}

/**
 * A motion bank: what it holds, the partition it routes through, and every motion in it.
 */
export function ArchiveOmfDescriptionView({
  "data-testid": dataTestId = "archive-omf-description-view",
  id,
  className,
  description,
}: IArchiveOmfDescriptionViewProps): ReactElement {
  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <ArchiveOmfBankSection bank={description.bank} parts={description.parts.length} />

      <ArchiveOmfPartitionSection parts={description.parts} />

      <ArchiveOmfMotionsSection motions={description.motions} />
    </ArchiveDescriptionLayout>
  );
}
