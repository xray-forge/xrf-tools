import { formatCount } from "@xrf/format";
import { ArchiveEfdDescription, ArchiveEfdPattern } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

import { describeResultRange, describeTermInputs, describeTermWeights } from "./ArchiveEfdView.utils";

interface IArchiveEfdViewProps extends BaseComponentProps {
  description: ArchiveEfdDescription;
}

/**
 * One trained evaluation function: what it answers with, what it reads to answer, and how its answer is built.
 */
export function ArchiveEfdView({
  "data-testid": dataTestId = "archive-efd-view",
  id,
  className,
  description,
}: IArchiveEfdViewProps): ReactElement {
  const { variableRanges, variableKinds, patterns } = description;

  return (
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Evaluation function"}>
        <EditorPanelProperty
          isInline
          label={"Registers as"}
          value={`Function ${description.functionType}`}
          caption={"The slot it claims in the engine's function table, which is how the game asks it anything"}
        />

        <EditorPanelProperty
          isInline
          label={"Answers between"}
          value={describeResultRange(description)}
          caption={"The band the trained table was built to answer in"}
        />

        <EditorPanelProperty
          isInline
          label={"Weights"}
          value={formatCount(description.weights)}
          caption={"The file stores no count of them; the terms below are what decides how many are read"}
        />

        <EditorPanelProperty isInline label={"Builder version"} value={`${description.builderVersion}`} />

        <EditorPanelProperty isInline label={"Data format"} value={`${description.dataFormat}`} />
      </EditorPanelSection>

      <EditorPanelSection
        title={`Inputs (${variableRanges.length})`}
        caption={"Numbered as the terms address them, each read from a function the engine already has"}
      >
        {variableRanges.map((range: number, index: number) => (
          <EditorPanelProperty
            isInline
            key={index}
            label={`Input ${index}`}
            value={`${formatCount(range)} ${range === 1 ? "bucket" : "buckets"}`}
            caption={`Read from function ${variableKinds[index]}`}
          />
        ))}
      </EditorPanelSection>

      <EditorPanelSection
        title={`Terms (${patterns.length})`}
        caption={"Summed to the answer, each contributing one weight per combination of what it reads"}
      >
        {patterns.map((pattern: ArchiveEfdPattern, index: number) => (
          <EditorPanelProperty
            isInline
            key={index}
            label={`Term ${index + 1}`}
            value={describeTermInputs(pattern)}
            caption={describeTermWeights(pattern)}
          />
        ))}
      </EditorPanelSection>
    </ReadingColumn>
  );
}
