import { BaseComponentProps } from "@xrf/ui/element-types";
import { Divider } from "@xrf/ui/material/divider";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement, ReactNode } from "react";

export interface IExportSectionProps extends BaseComponentProps {
  isLast?: boolean;
  title: string;
  children: ReactNode;
}

export function ExportSection({ children, isLast = false, title }: IExportSectionProps): ReactElement {
  return (
    <section className={cn(isLast ? null : "pb-5")}>
      <Typography className={"mb-2"} variant={"subtitle2"}>
        {title}
      </Typography>

      {children}

      {isLast ? null : <Divider className={"mt-5"} />}
    </section>
  );
}
