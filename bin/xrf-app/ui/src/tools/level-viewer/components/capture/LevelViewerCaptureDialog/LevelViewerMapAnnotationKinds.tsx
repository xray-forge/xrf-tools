import { LevelObjectKind, LevelObjectKindEntry } from "@xrf/ipc/types/xrf-app-levels";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import {
  ILevelObjectGroupEntry,
  LEVEL_OBJECT_GROUPS,
  listLevelObjectGroupKinds,
} from "@/core/level/lib/objects/level-object-kinds";

interface ILevelViewerMapAnnotationKindsProps extends BaseComponentProps {
  /** The kinds drawn. */
  kinds: ReadonlyArray<LevelObjectKind>;
  onChange: (kinds: ReadonlyArray<LevelObjectKind>, isShown: boolean) => void;
}

/**
 * The kinds of level object a map's annotations draw, by the kind table's families: a family of several kinds by its
 * own switch over them, mixed while only some are drawn, its kinds beneath; a family of one by its kind alone.
 */
export function LevelViewerMapAnnotationKinds({
  "data-testid": dataTestId = "level-viewer-map-annotation-kinds",
  id,
  className,
  kinds,
  onChange,
}: ILevelViewerMapAnnotationKindsProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("columns-4 gap-x-6", className)}>
      {LEVEL_OBJECT_GROUPS.map((group: ILevelObjectGroupEntry) => {
        const entries: Array<LevelObjectKindEntry> = listLevelObjectGroupKinds(group.group);
        const members: Array<LevelObjectKind> = entries.map((entry: LevelObjectKindEntry) => entry.kind);
        const drawn: number = members.filter((kind: LevelObjectKind) => kinds.includes(kind)).length;

        return (
          <div key={group.group} className={"break-inside-avoid"}>
            {entries.length > 1 ? (
              <CheckboxFormRow
                label={group.label}
                isChecked={drawn === members.length}
                isIndeterminate={drawn > 0 && drawn < members.length}
                onChange={(isShown: boolean) => onChange(members, isShown)}
              />
            ) : null}
            <div className={cn(entries.length > 1 && "pl-6")}>
              {entries.map((entry: LevelObjectKindEntry) => (
                <CheckboxFormRow
                  key={entry.kind}
                  label={entry.label}
                  isChecked={kinds.includes(entry.kind)}
                  onChange={(isShown: boolean) => onChange([entry.kind], isShown)}
                />
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}
