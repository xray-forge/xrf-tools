import { ReactElement } from "react";

import { ArchiveAnimationChannel } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveAnimationChannelRow } from "../ArchiveAnimationChannelRow";

interface IArchiveAnmChannelsSectionProps extends BaseComponentProps {
  channels: Array<ArchiveAnimationChannel>;
}

/**
 * The six channels an object motion drives, in the order the format stores them.
 */
export function ArchiveAnmChannelsSection({
  "data-testid": dataTestId = "archive-anm-channels-section",
  id,
  className,
  channels,
}: IArchiveAnmChannelsSectionProps): ReactElement {
  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Channels"}
      caption={"Position in engine units and rotation in radians, each keyed on its own envelope"}
    >
      {channels.map((channel: ArchiveAnimationChannel) => (
        <ArchiveAnimationChannelRow key={channel.name} channel={channel} />
      ))}
    </EditorPanelSection>
  );
}
