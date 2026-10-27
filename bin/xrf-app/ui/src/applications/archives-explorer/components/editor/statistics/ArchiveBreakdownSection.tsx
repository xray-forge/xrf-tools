import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DetailSection } from "@xrf/ui/layout/DetailSection";
import { IStatBreakdownRow, StatBreakdownTable } from "@xrf/ui/stats/StatBreakdownTable";
import { StatMeasureToggle } from "@xrf/ui/stats/StatMeasureToggle";
import { ReactElement, ReactNode } from "react";

import { IArchiveStatisticsView } from "./archive-statistics-section";

export interface IArchiveBreakdownSectionProps extends BaseComponentProps {
  title: string;
  description: string;
  fact?: Nullable<string>;
  rows: Array<IStatBreakdownRow>;
  view: IArchiveStatisticsView;
  /** Keeps the rows in the order the report gave them. See {@link StatBreakdownTable}. */
  isPreordered?: boolean;
  /** Figures stated above the table, where the rows alone do not carry the answer. */
  children?: ReactNode;
}

/**
 * A titled breakdown: the chrome, the measurement toggle, and the table, in the one arrangement every breakdown uses.
 */
export function ArchiveBreakdownSection({
  "data-testid": dataTestId,
  id,
  className,
  title,
  description,
  fact = null,
  rows,
  view,
  isPreordered,
  children,
}: IArchiveBreakdownSectionProps): ReactElement {
  return (
    <DetailSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={title}
      description={description}
      fact={fact}
      action={<StatMeasureToggle measure={view.measure} onChange={view.onMeasureChange} />}
    >
      {children}

      <StatBreakdownTable rows={rows} measure={view.measure} isPreordered={isPreordered} />
    </DetailSection>
  );
}
