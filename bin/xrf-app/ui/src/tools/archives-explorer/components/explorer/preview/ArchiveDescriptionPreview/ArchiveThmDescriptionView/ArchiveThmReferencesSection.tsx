import { formatNumber } from "@xrf/format";
import {
  ArchiveDescribeScope,
  ArchiveReference,
  ArchiveThmBump,
  ArchiveThmDetail,
} from "@xrf/ipc/types/xrf-app-archives";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { NOT_DECLARED } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionReference } from "../ArchiveDescriptionReference";
import { describeDetailUsage } from "./ArchiveThmDescriptionView.utils";

interface IArchiveThmReferencesSectionProps extends BaseComponentProps {
  bump: Nullable<ArchiveThmBump>;
  detail: Nullable<ArchiveThmDetail>;
  externalNormalMap: Nullable<ArchiveReference>;
  scope: ArchiveDescribeScope;
}

/**
 * The textures a descriptor names, each resolved against the subject being browsed.
 */
export function ArchiveThmReferencesSection({
  "data-testid": dataTestId = "archive-thm-references-section",
  id,
  className,
  bump,
  detail,
  externalNormalMap,
  scope,
}: IArchiveThmReferencesSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"References"}>
      {bump ? (
        <>
          <EditorPanelProperty
            isInline
            label={"Bump mode"}
            value={bump.modeLabel}
            caption={bump.isUsed ? null : "The engine does not resolve a bump texture for this descriptor"}
          />

          {bump.texture ? (
            <ArchiveDescriptionReference label={"Bump texture"} reference={bump.texture} scope={scope} />
          ) : (
            <EditorPanelProperty isInline label={"Bump texture"} value={"Names none"} />
          )}

          <EditorPanelProperty
            isInline
            label={"Virtual height"}
            value={formatNumber(bump.virtualHeight, 3)}
            caption={"Read when the pair is generated, never at runtime"}
          />
        </>
      ) : (
        <EditorPanelProperty isInline label={"Bump"} value={NOT_DECLARED} />
      )}

      {detail ? (
        <>
          {detail.texture ? (
            <ArchiveDescriptionReference label={"Detail texture"} reference={detail.texture} scope={scope} />
          ) : (
            <EditorPanelProperty isInline label={"Detail texture"} value={"Names none"} />
          )}

          <EditorPanelProperty isInline label={"Detail scale"} value={formatNumber(detail.scale, 3)} />

          <EditorPanelProperty isInline label={"Detail applied by"} value={describeDetailUsage(detail)} />
        </>
      ) : (
        <EditorPanelProperty isInline label={"Detail"} value={NOT_DECLARED} />
      )}

      {externalNormalMap ? (
        <ArchiveDescriptionReference label={"External normal map"} reference={externalNormalMap} scope={scope} />
      ) : null}
    </EditorPanelSection>
  );
}
