import { Alert, Box, Button, List, Stack, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { TextureFormatRow } from "@/applications/textures-editor/components/panels/TextureFormatsPanel/TextureFormatRow";
import { TextureEncodingService } from "@/applications/textures-editor/services/encoding";
import { TextureDescription, TextureEncodingComparison, TextureEncodingReport } from "@/core/bindings/types/xrf-app";
import {
  EditorPanel,
  EditorPanelEmpty,
  EditorPanelProperty,
  EditorPanelSection,
} from "@/core/shell/editor/EditorPanel";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

/**
 * What every format would cost this texture, and which one a save would write.
 */
export function TextureFormatsPanel({
  "data-testid": dataTestId = "texture-formats-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const encodingService: TextureEncodingService = useInjection(TextureEncodingService);

  const description: Nullable<TextureDescription> = selectionService.selected.value;
  const comparison: Nullable<TextureEncodingComparison> = encodingService.comparison;

  const onCompare = useCallback(() => void encodingService.run("kaiser"), [encodingService]);

  if (!description) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Formats"}>
        <EditorPanelEmpty label={"No texture selected. What each format would cost it shows here."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Formats"}>
      <EditorPanelSection title={"Current"} isFirst>
        <EditorPanelProperty
          label={"Format"}
          value={comparison?.current.label ?? description.base?.shape?.format ?? "-"}
        />
        <EditorPanelProperty
          label={"On disk"}
          value={comparison ? formatBytes(comparison.current.fileBytes) : formatBytes(description.base?.size ?? 0)}
        />
        {comparison ? (
          <EditorPanelProperty label={"Uploaded"} value={formatBytes(comparison.current.gpuBytes)} />
        ) : null}
      </EditorPanelSection>

      <EditorPanelSection
        title={"Candidates"}
        caption={"Every figure is what a re-encode adds on top of the file as it stands"}
      >
        <Stack direction={"row"} spacing={1} sx={{ mb: 1 }}>
          <Button
            data-testid={"texture-formats-compare"}
            size={"small"}
            variant={"contained"}
            disabled={encodingService.compare.isRunning}
            onClick={onCompare}
          >
            {comparison ? "Weigh again" : "Weigh formats"}
          </Button>

          {encodingService.compare.isRunning ? (
            <Button data-testid={"texture-formats-cancel"} size={"small"} onClick={encodingService.compare.cancel}>
              Stop
            </Button>
          ) : null}
        </Stack>

        {encodingService.compare.error ? <Alert severity={"error"}>{encodingService.compare.error}</Alert> : null}

        {comparison ? (
          <List dense disablePadding data-testid={"texture-formats-list"}>
            {comparison.candidates.map((candidate: TextureEncodingReport) => (
              <TextureFormatRow
                key={candidate.format}
                data-testid={`texture-format-${candidate.format}`}
                candidate={candidate}
                isChosen={encodingService.chosen === candidate.format}
                onChoose={() => void encodingService.choose(candidate.format)}
              />
            ))}
          </List>
        ) : (
          <Typography variant={"caption"} color={"text.secondary"}>
            {encodingService.compare.isRunning
              ? "Encoding each candidate from the same reduced levels, cheapest first."
              : "Nothing weighed yet. BC7 alone takes over a second on a large texture, so this is asked for."}
          </Typography>
        )}

        {comparison?.outcome === "cancelled" ? (
          <Alert severity={"info"} sx={{ mt: 1 }}>
            Stopped before every format was weighed. The ones listed are real measurements and can still be saved.
          </Alert>
        ) : null}
      </EditorPanelSection>

      {encodingService.chosenReport ? (
        <EditorPanelSection title={"Pending"}>
          <Box>
            <Alert severity={"info"}>
              {`Saving writes this texture as ${encodingService.chosenReport.label}. The descriptor's own format is ` +
                "updated with it, except for BC7, which ETFormat cannot name."}
            </Alert>
          </Box>
        </EditorPanelSection>
      ) : null}
    </EditorPanel>
  );
}
