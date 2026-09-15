import { Typography } from "@mui/material";
import { ReactElement, useMemo, useState } from "react";

import { ArchiveOmfMotion } from "@/core/ipc/types/xrf-app";
import { EditorFilterInput } from "@/core/shell/editor/EditorFilterInput";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveOmfMotionRow } from "./ArchiveOmfMotionRow";

interface IArchiveOmfMotionsSectionProps extends BaseComponentProps {
  motions: Array<ArchiveOmfMotion>;
}

/**
 * Every motion the bank holds, in the order it declares them.
 */
export function ArchiveOmfMotionsSection({
  "data-testid": dataTestId = "archive-omf-motions-section",
  id,
  className,
  motions,
}: IArchiveOmfMotionsSectionProps): ReactElement {
  const [filter, setFilter] = useState<string>("");

  const matched: Array<ArchiveOmfMotion> = useMemo(() => {
    const needle: string = filter.trim().toLowerCase();

    return needle ? motions.filter((motion: ArchiveOmfMotion) => motion.name.toLowerCase().includes(needle)) : motions;
  }, [filter, motions]);

  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={filter.trim() ? `Motions (${matched.length} of ${motions.length})` : `Motions (${motions.length})`}
      caption={"In the order the bank declares them, which is the order the engine pairs them by"}
    >
      <EditorFilterInput
        query={filter}
        placeholder={"Filter motions"}
        ariaLabel={"Filter motions"}
        onQueryChange={setFilter}
        sx={{ marginBottom: 1 }}
      />

      {matched.length ? (
        matched.map((motion: ArchiveOmfMotion, index: number) => (
          <ArchiveOmfMotionRow key={`${index}-${motion.name}`} motion={motion} />
        ))
      ) : (
        <Typography variant={"body2"} sx={{ color: "text.disabled" }}>
          No motion of this bank is named that.
        </Typography>
      )}
    </EditorPanelSection>
  );
}
