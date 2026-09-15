import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveReference, ArchiveThmBump, ArchiveThmDetail } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatNumber } from "@/lib/format/number";
import { Nullable } from "@/lib/types/general";

import { NOT_DECLARED } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionReference } from "../ArchiveDescriptionReference";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
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
          <ArchiveDescriptionRow
            label={"Bump mode"}
            value={bump.modeLabel}
            caption={bump.isUsed ? null : "The engine does not resolve a bump texture for this descriptor"}
          />

          {bump.texture ? (
            <ArchiveDescriptionReference label={"Bump texture"} reference={bump.texture} scope={scope} />
          ) : (
            <ArchiveDescriptionRow label={"Bump texture"} value={"Names none"} />
          )}

          <ArchiveDescriptionRow
            label={"Virtual height"}
            value={formatNumber(bump.virtualHeight, 3)}
            caption={"Read when the pair is generated, never at runtime"}
          />
        </>
      ) : (
        <ArchiveDescriptionRow label={"Bump"} value={NOT_DECLARED} />
      )}

      {detail ? (
        <>
          {detail.texture ? (
            <ArchiveDescriptionReference label={"Detail texture"} reference={detail.texture} scope={scope} />
          ) : (
            <ArchiveDescriptionRow label={"Detail texture"} value={"Names none"} />
          )}

          <ArchiveDescriptionRow label={"Detail scale"} value={formatNumber(detail.scale, 3)} />

          <ArchiveDescriptionRow label={"Detail applied by"} value={describeDetailUsage(detail)} />
        </>
      ) : (
        <ArchiveDescriptionRow label={"Detail"} value={NOT_DECLARED} />
      )}

      {externalNormalMap ? (
        <ArchiveDescriptionReference label={"External normal map"} reference={externalNormalMap} scope={scope} />
      ) : null}
    </EditorPanelSection>
  );
}
