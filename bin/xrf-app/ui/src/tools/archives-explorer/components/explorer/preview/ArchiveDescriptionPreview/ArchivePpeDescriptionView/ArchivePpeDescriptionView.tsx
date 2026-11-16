import { ArchiveDescribeScope, ArchivePpeColor, ArchivePpeDescription } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

import { ArchivePpeColorSection } from "./ArchivePpeColorSection";
import { ArchivePpeEffectSection } from "./ArchivePpeEffectSection";
import { ArchivePpeValuesSection } from "./ArchivePpeValuesSection";

interface IArchivePpeDescriptionViewProps extends BaseComponentProps {
  description: ArchivePpeDescription;
  scope: ArchiveDescribeScope;
}

/**
 * A post-process effect: how long it runs, and what each of its parameters does over that time.
 */
export function ArchivePpeDescriptionView({
  "data-testid": dataTestId = "archive-ppe-description-view",
  id,
  className,
  description,
  scope,
}: IArchivePpeDescriptionViewProps): ReactElement {
  return (
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <ArchivePpeEffectSection description={description} scope={scope} />

      {description.colors.map((color: ArchivePpeColor) => (
        <ArchivePpeColorSection key={color.name} color={color} />
      ))}

      <ArchivePpeValuesSection values={description.values} colorMap={description.colorMap} />
    </ReadingColumn>
  );
}
