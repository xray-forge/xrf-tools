import { ArchiveAnimationChannel } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

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
