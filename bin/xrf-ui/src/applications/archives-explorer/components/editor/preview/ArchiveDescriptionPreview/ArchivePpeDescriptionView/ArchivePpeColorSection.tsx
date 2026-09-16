import { ReactElement } from "react";

import { ArchiveAnimationChannel, ArchivePpeColor } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
