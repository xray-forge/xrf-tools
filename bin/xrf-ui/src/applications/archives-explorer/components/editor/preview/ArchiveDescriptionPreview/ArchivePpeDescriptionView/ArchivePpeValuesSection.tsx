import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { ArchiveAnimationChannel, ArchivePpeColorMap } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveAnimationChannelRow } from "../ArchiveAnimationChannelRow";

interface IArchivePpeValuesSectionProps extends BaseComponentProps {
  values: Array<ArchiveAnimationChannel>;
  /** The grading's influence joins the scalars, because it is one too - it just arrived a version later. */
  colorMap: Nullable<ArchivePpeColorMap>;
}

/**
 * The scalar parameters an effect animates, in the order the format stores them.
 */
export function ArchivePpeValuesSection({
  "data-testid": dataTestId = "archive-ppe-values-section",
  id,
  className,
  values,
  colorMap,
}: IArchivePpeValuesSectionProps): ReactElement {
  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Parameters"}
      caption={"Every effect stores all of them; a shipped one keys three or four"}
    >
      {values.map((channel: ArchiveAnimationChannel) => (
        <ArchiveAnimationChannelRow key={channel.name} channel={channel} />
      ))}

      {colorMap ? <ArchiveAnimationChannelRow channel={colorMap.influence} /> : null}
    </EditorPanelSection>
  );
}
