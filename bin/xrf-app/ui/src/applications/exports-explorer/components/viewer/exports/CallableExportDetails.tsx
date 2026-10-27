import { ABSENT_VALUE } from "@xrf/format";
import { ExportParameterDescriptor } from "@xrf/ipc/types/xrf-export";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Table, TableBody, TableCell, TableHead, TableRow } from "@xrf/ui/material/table";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement } from "react";

import { TCallableExportDescriptor } from "@/applications/exports-explorer/lib/export-descriptors";

import { ExportSection } from "./ExportSection";

export interface ICallableExportDetailsProps extends BaseComponentProps {
  declaration: TCallableExportDescriptor;
}

export function CallableExportDetails({ declaration }: ICallableExportDetailsProps): ReactElement {
  return (
    <>
      <ExportSection title={"Parameters"}>
        {declaration.parameters.length ? (
          <Table size={"small"} aria-label={"Export parameters"}>
            <TableHead>
              <TableRow>
                <TableCell>Name</TableCell>
                <TableCell>Type</TableCell>
                <TableCell>Description</TableCell>
              </TableRow>
            </TableHead>

            <TableBody>
              {declaration.parameters.map((parameter: ExportParameterDescriptor) => (
                <TableRow key={parameter.name}>
                  <TableCell className={"monospace"}>
                    {parameter.name}
                    {parameter.isOptional ? "?" : ""}
                  </TableCell>

                  <TableCell className={"monospace wrap-anywhere"}>{parameter.typing}</TableCell>
                  <TableCell>{parameter.description ?? ABSENT_VALUE}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        ) : (
          <Typography className={"text-text-secondary"} variant={"body2"}>
            No parameters.
          </Typography>
        )}
      </ExportSection>

      <ExportSection title={"Returns"}>
        <Typography className={"monospace wrap-anywhere"} variant={"body2"}>
          {declaration.returns.typing}
        </Typography>

        {declaration.returns.description ? (
          <Typography className={"mt-1.5 whitespace-pre-wrap"} variant={"body2"}>
            {declaration.returns.description}
          </Typography>
        ) : null}
      </ExportSection>
    </>
  );
}
