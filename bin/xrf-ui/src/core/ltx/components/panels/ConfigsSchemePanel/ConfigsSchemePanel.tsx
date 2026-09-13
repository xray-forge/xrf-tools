import { Box, Divider } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect } from "react";

import { LtxSchemeFieldReport, LtxSectionSchemeReport } from "@/core/ipc/types/xrf-ltx-inspect";
import { ConfigsSchemeBinding } from "@/core/ltx/components/panels/ConfigsSchemePanel/ConfigsSchemeBinding";
import { ConfigsSchemeFieldRow } from "@/core/ltx/components/panels/ConfigsSchemePanel/ConfigsSchemeFieldRow";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsSchemeService } from "@/core/ltx/services/scheme";
import { EditorPanel, EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * What judges the selected section, and how the section measures against it.
 */
export function ConfigsSchemePanel({
  "data-testid": dataTestId = "configs-scheme-panel",
}: BaseComponentProps): ReactElement {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);
  const schemeService: ConfigsSchemeService = useInjection(ConfigsSchemeService);

  const entry: Nullable<string> = documentService.entry;
  const selected: Nullable<string> = documentService.selectedSection;

  useEffect(() => {
    if (entry && selected) {
      void schemeService.read(entry, selected);
    } else {
      schemeService.clear();
    }
  }, [entry, schemeService, selected]);

  if (schemeService.report.isFailed) {
    return (
      <ErrorState
        data-testid={dataTestId}
        title={"Cannot explain this section"}
        description={schemeService.report.error?.message ?? "The read failed."}
        onRetry={() => void (entry && selected && schemeService.read(entry, selected))}
      />
    );
  }

  if (!selected || !entry) {
    return (
      <EditorPanel data-testid={dataTestId} title={"Scheme"}>
        <EditorPanelEmpty
          label={
            entry
              ? "Pick a section to see what it is judged by."
              : "Nothing resolves this config, so no scheme is bound to its sections."
          }
        />
      </EditorPanel>
    );
  }

  if (schemeService.report.isLoading) {
    return <DelayedProgress data-testid={dataTestId} />;
  }

  const report: Nullable<LtxSectionSchemeReport> = schemeService.report.value;

  if (!report) {
    return (
      <EditorPanel data-testid={dataTestId} title={"Scheme"}>
        <EditorPanelEmpty label={`${selected} is not a section of ${entry}.`} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} title={"Scheme"}>
      <ConfigsSchemeBinding report={report} />

      <Divider />

      {report.fields.length ? (
        <Box
          data-testid={"configs-scheme-fields"}
          sx={{ padding: 1.5, display: "flex", flexDirection: "column", gap: 1 }}
        >
          {report.fields.map((field: LtxSchemeFieldReport) => (
            <ConfigsSchemeFieldRow
              key={field.name}
              field={field}
              isJudged={report.isDeclared}
              isStrict={report.isStrict}
            />
          ))}
        </Box>
      ) : (
        <EditorPanelEmpty label={"This section holds no fields."} />
      )}
    </EditorPanel>
  );
}
