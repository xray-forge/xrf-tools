import { formatBytes, formatCount, formatPercent } from "@xrf/format";
import { ReactElement } from "react";

import { RenderMemoryReport, RenderMemorySection } from "@/core/ipc/types/xrf-renderer";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelRendererMemorySectionProps extends BaseComponentProps {
  /** What the renderer holds on the GPU. */
  memory: RenderMemoryReport;
}

/**
 * @param section - A share of what the GPU allocator holds allocated.
 * @param allocated - All it holds allocated, or null where it reports none.
 * @returns Its bytes, its share of all of it, and how many resources make it up.
 */
function formatSection(section: RenderMemorySection, allocated: number | null): string {
  const share: string = allocated ? ` · ${formatPercent(section.bytes / allocated, 1)}` : "";

  return `${formatBytes(section.bytes)}${share} · ${formatCount(section.count)}`;
}

/**
 * What the GPU allocator holds allocated by the names the renderer made its resources under, the largest first, the
 * smallest summed into one; that it reports none where its backend does not.
 */
export function LevelRendererMemorySection({
  "data-testid": dataTestId = "level-renderer-memory-section",
  id,
  className,
  memory,
}: ILevelRendererMemorySectionProps): ReactElement {
  const sections: ReadonlyArray<RenderMemorySection> = memory.sections ?? [];

  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"GPU memory"}>
      {sections.length === 0 ? (
        <EditorPanelProperty label={"Sections"} value={"Not reported"} />
      ) : (
        sections.map((section: RenderMemorySection) => (
          <EditorPanelProperty
            key={section.name}
            label={section.name}
            value={formatSection(section, memory.allocated)}
          />
        ))
      )}
    </EditorPanelSection>
  );
}
