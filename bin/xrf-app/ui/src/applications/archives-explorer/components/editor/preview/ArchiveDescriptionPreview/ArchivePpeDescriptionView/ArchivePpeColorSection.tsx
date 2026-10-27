import { ArchiveAnimationChannel, ArchivePpeColor } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EditorPanelSection } from "@xrf/ui/shell/editor/EditorPanel/EditorPanelSection";
import { ReactElement } from "react";

import { ArchiveAnimationChannelRow } from "../ArchiveAnimationChannelRow";
import { describeColor } from "./ArchivePpeDescriptionView.utils";

interface IArchivePpeColorSectionProps extends BaseComponentProps {
  color: ArchivePpeColor;
}

/**
 * One colour parameter, and the three channels the engine assembles it from.
 */
export function ArchivePpeColorSection({
  "data-testid": dataTestId = "archive-ppe-color-section",
  id,
  className,
  color,
}: IArchivePpeColorSectionProps): ReactElement {
  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={color.name}
      caption={describeColor(color)}
    >
      {color.channels.map((channel: ArchiveAnimationChannel) => (
        <ArchiveAnimationChannelRow key={channel.name} channel={channel} />
      ))}
    </EditorPanelSection>
  );
}
