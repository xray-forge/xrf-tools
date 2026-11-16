import { ArchiveDetailLibraryDescription } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

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
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <ArchiveDetailLayerSection description={description} />

      <ArchiveDetailObjectsSection entries={description.entries} />
    </ReadingColumn>
  );
}
