import { ReactElement } from "react";

import { ArchiveAnmChannel } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeChannelDetail, describeChannelKeys } from "./ArchiveAnmDescriptionView.utils";

interface IArchiveAnmChannelsSectionProps extends BaseComponentProps {
  channels: Array<ArchiveAnmChannel>;
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
      {channels.map((channel: ArchiveAnmChannel) => (
        <ArchiveDescriptionRow
          key={channel.name}
          label={channel.name}
          value={describeChannelKeys(channel)}
          caption={describeChannelDetail(channel)}
        />
      ))}
    </EditorPanelSection>
  );
}
