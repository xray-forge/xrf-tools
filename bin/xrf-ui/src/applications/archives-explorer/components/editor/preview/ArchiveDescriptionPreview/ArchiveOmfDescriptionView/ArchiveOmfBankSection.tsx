import { ReactElement } from "react";

import { ArchiveOmfBank } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { MOTION_SAMPLE_FPS } from "@/core/visuals/lib/visual-motion";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { formatSeconds } from "./ArchiveOmfDescriptionView.utils";

interface IArchiveOmfBankSectionProps extends BaseComponentProps {
  bank: ArchiveOmfBank;
  /** Parts of the partition, counted here because the section says how much there is of everything. */
  parts: number;
}

/**
 * What the bank holds, taken over the whole of it.
 */
export function ArchiveOmfBankSection({
  "data-testid": dataTestId = "archive-omf-bank-section",
  id,
  className,
  bank,
  parts,
}: IArchiveOmfBankSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Bank"} isFirst>
      <ArchiveDescriptionRow
        label={"Motions"}
        value={`${bank.motions}`}
        caption={bank.effects ? `${bank.effects} of them effects, played on a bone rather than on a part` : null}
      />

      <ArchiveDescriptionRow
        label={"Length"}
        value={formatSeconds(bank.durationSeconds)}
        caption={`${bank.frames} frames together, at the format's fixed ${MOTION_SAMPLE_FPS} fps`}
      />

      <ArchiveDescriptionRow
        label={"Partition"}
        value={`${parts} ${parts === 1 ? "part" : "parts"} · ${bank.bones} bones`}
        caption={"The skeleton this bank was authored against"}
      />

      <ArchiveDescriptionRow
        label={"Version"}
        value={`${bank.version}`}
        caption={bank.carriesMarks ? null : "Below version 4, which is where motion marks were added"}
      />

      {bank.markedMotions ? (
        <ArchiveDescriptionRow
          label={"Marked motions"}
          value={`${bank.markedMotions}`}
          caption={"Carry at least one mark, which is what the inverse-kinematics controller reads"}
        />
      ) : null}

      {bank.replacedFalloffs ? (
        <ArchiveDescriptionRow
          label={"Replaced falloffs"}
          value={`${bank.replacedFalloffs}`}
          caption={
            "The engine rewrites the falloff of any motion that is not an effect and does not declare one below its " +
            "accrue, so the blend below is what plays rather than what the file says"
          }
        />
      ) : null}

      {bank.divergingLabels ? (
        <ArchiveDescriptionRow
          label={"Stale labels"}
          value={`${bank.divergingLabels}`}
          caption={"Payloads still carrying a name that is not the motion's; release playback never reads them"}
        />
      ) : null}
    </EditorPanelSection>
  );
}
