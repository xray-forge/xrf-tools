import { ReactElement } from "react";

import { ArchiveAnimationChannel } from "@/core/ipc/types/xrf-app";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeChannelDetail, describeChannelKeys } from "./ArchiveAnimationChannelRow.utils";

interface IArchiveAnimationChannelRowProps extends BaseComponentProps {
  channel: ArchiveAnimationChannel;
  /** Shown in place of the channel's own name, for a surface that already says what the channel is part of. */
  label?: string;
}

/**
 * One envelope of any format built on `CEnvelope`: what it is keyed for, and what qualifies those keys.
 */
export function ArchiveAnimationChannelRow({
  "data-testid": dataTestId = "archive-animation-channel-row",
  id,
  className,
  channel,
  label,
}: IArchiveAnimationChannelRowProps): ReactElement {
  return (
    <ArchiveDescriptionRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label ?? channel.name}
      value={describeChannelKeys(channel)}
      caption={describeChannelDetail(channel)}
    />
  );
}
