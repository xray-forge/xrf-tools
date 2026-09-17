import { Table, TableBody, TableCell, TableHead, TableRow, Typography } from "@mui/material";
import { ReactElement } from "react";

import { TCallableExportDescriptor } from "@/core/exports";
import { ExportParameterDescriptor } from "@/core/ipc/types/xrf-export";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
                  <TableCell>{parameter.description ?? "—"}</TableCell>
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
