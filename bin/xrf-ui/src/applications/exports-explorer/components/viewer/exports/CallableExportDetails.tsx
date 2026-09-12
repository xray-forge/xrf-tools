import { Table, TableBody, TableCell, TableHead, TableRow, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ExportParameterDescriptor } from "@/core/bindings/types/xrf-export";
import { TCallableExportDescriptor } from "@/core/exports";
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

                  <TableCell className={"monospace"} sx={{ overflowWrap: "anywhere" }}>
                    {parameter.typing}
                  </TableCell>
                  <TableCell>{parameter.description ?? "—"}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        ) : (
          <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
            No parameters.
          </Typography>
        )}
      </ExportSection>

      <ExportSection title={"Returns"}>
        <Typography variant={"body2"} className={"monospace"} sx={{ overflowWrap: "anywhere" }}>
          {declaration.returns.typing}
        </Typography>

        {declaration.returns.description ? (
          <Typography variant={"body2"} sx={{ marginTop: 0.75, whiteSpace: "pre-wrap" }}>
            {declaration.returns.description}
          </Typography>
        ) : null}
      </ExportSection>
    </>
  );
}
