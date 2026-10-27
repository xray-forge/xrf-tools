import { ArchiveAnmDescription } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { ArchiveAnmChannelsSection } from "./ArchiveAnmChannelsSection";
import { ArchiveAnmMotionSection } from "./ArchiveAnmMotionSection";

interface IArchiveAnmDescriptionViewProps extends BaseComponentProps {
  description: ArchiveAnmDescription;
}

/**
 * An object motion: how long the engine plays it, and what each of its channels does.
 */
export function ArchiveAnmDescriptionView({
  "data-testid": dataTestId = "archive-anm-description-view",
  id,
  className,
  description,
}: IArchiveAnmDescriptionViewProps): ReactElement {
  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <ArchiveAnmMotionSection description={description} />

      <ArchiveAnmChannelsSection channels={description.channels} />
    </ArchiveDescriptionLayout>
  );
}
