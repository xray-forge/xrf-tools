import { ReactElement, ReactNode } from "react";

import { DetailSection } from "@/core/ui/layout/DetailSection";
import { IStatBreakdownRow, StatBreakdownTable } from "@/core/ui/stats/StatBreakdownTable";
import { StatMeasureToggle } from "@/core/ui/stats/StatMeasureToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

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
