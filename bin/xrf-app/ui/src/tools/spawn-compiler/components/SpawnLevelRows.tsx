import { SpawnListedLevel } from "@xrf/ipc/types/xrf-spawn-builder";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ReactElement } from "react";

import { toggleSpawnLevel } from "@/tools/spawn-compiler/lib/spawn-request";

interface ISpawnLevelRowsProps {
  /** The levels built in the levels directory, in the order the game lists them. */
  levels: ReadonlyArray<SpawnListedLevel>;
  /** The levels ticked, by directory name. */
  chosen: ReadonlyArray<string>;
  isDisabled: boolean;
  onChange: (chosen: ReadonlyArray<string>) => void;
}

/**
 * One box per built level, in the order the game lists them, each described by the id and caption the game gives it.
 */
export function SpawnLevelRows({ levels, chosen, isDisabled, onChange }: ISpawnLevelRowsProps): ReactElement {
  return (
    <>
      {levels.map((level: SpawnListedLevel) => (
        <CheckboxFormRow
          key={level.name}
          label={level.name}
          description={`Level ${level.id}, connected to as '${level.caption}'`}
          isChecked={chosen.includes(level.name)}
          isDisabled={isDisabled}
          onChange={(isChecked: boolean) => onChange(toggleSpawnLevel(levels, chosen, level.name, isChecked))}
        />
      ))}
    </>
  );
}
