import { ReactElement } from "react";

import { ArchiveDetailLibraryDescription } from "@/core/ipc/types/xrf-app";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { ArchiveDetailLayerSection } from "./ArchiveDetailLayerSection";
import { ArchiveDetailObjectsSection } from "./ArchiveDetailObjectsSection";

interface IArchiveDetailLibraryDescriptionViewProps extends BaseComponentProps {
  description: ArchiveDetailLibraryDescription;
}

/**
 * A level's detail layer: how much of the level it dresses, and what it dresses it with.
 */
export function ArchiveDetailLibraryDescriptionView({
  "data-testid": dataTestId = "archive-detail-library-description-view",
  id,
  className,
  description,
}: IArchiveDetailLibraryDescriptionViewProps): ReactElement {
  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <ArchiveDetailLayerSection description={description} />

      <ArchiveDetailObjectsSection entries={description.entries} />
    </ArchiveDescriptionLayout>
  );
}
