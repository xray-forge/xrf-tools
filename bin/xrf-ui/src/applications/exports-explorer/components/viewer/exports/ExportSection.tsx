import { Divider, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
