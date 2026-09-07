import { ReactElement } from "react";

import {
  EditorPanel,
  EditorPanelEmpty,
  EditorPanelProperty,
  EditorPanelSection,
} from "@/core/shell/editor/EditorPanel";
import { ISpawnRowSelection, SpawnFileService } from "@/core/spawn/services";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { formatSpawnRowDetailsValue } from "./SpawnRowDetailsPanel.utils";

export interface ISpawnRowDetailsPanelProps extends BaseComponentProps {
  spawnFileService: SpawnFileService;
}

/**
 * Everything about the selected row.
 *
 * The tables keep their columns terse because this exists: a spawn record has more fields than fit on a
 * screen, and most of them are only wanted once you have found the row you care about.
 */
export function SpawnRowDetailsPanel({
  "data-testid": dataTestId = "spawn-row-details-panel",
  id,
  className,
  spawnFileService,
}: ISpawnRowDetailsPanelProps): ReactElement {
  const selection: Nullable<ISpawnRowSelection> = spawnFileService.selectedRow;

  if (!selection) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Nothing selected"}>
        <EditorPanelEmpty label={"Pick a row in any chunk table to inspect it here."} />
      </EditorPanel>
    );
  }

  const entries: Array<[string, unknown]> = Object.entries(selection.row);

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={selection.source}>
      <EditorPanelSection title={"Properties"} isFirst>
        {entries.map(([key, value]: [string, unknown]) => (
          <EditorPanelProperty key={key} label={key} value={formatSpawnRowDetailsValue(value)} isMonospace />
        ))}
      </EditorPanelSection>
    </EditorPanel>
  );
}
