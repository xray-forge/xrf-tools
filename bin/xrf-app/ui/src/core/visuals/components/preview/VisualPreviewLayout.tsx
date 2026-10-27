import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { EditorFileHeader } from "@xrf/ui/editor/file-header";
import { EditorLayout } from "@xrf/ui/editor/layout";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { AccountTreeIcon } from "@xrf/ui/icons/account-tree";
import { DelayedProgress } from "@xrf/ui/layout/delayed-progress";
import { cn } from "@xrf/ui/theme";
import { ReactElement, ReactNode, useMemo } from "react";

import { isAlphaRenderSurfaceDraw } from "@/core/render/lib/surface/render-surface-draw";
import { useShellPanels, useShellStatus } from "@/core/shell/contributions";
import { IEditorPanel } from "@/core/shell/editor-shell";
import { VisualPreviewEmpty, VisualPreviewToolbar } from "@/core/visuals/components/preview";
import { VisualPreviewViewport } from "@/core/visuals/components/preview/VisualPreviewViewport";
import { IVisualRenderSource, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { countVisualTriangles, IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { VisualViewService } from "@/core/visuals/services/visual-view";

interface IVisualPreviewLayoutProps extends BaseComponentProps {
  /** Shown in the toolbar beside the view toggles, usually where the model came from. */
  subtitle?: ReactNode;
  /** What the open visual is called. Its presence is what draws the file header over the viewport. */
  name?: Nullable<string>;
  /** Published as a left panel when given. Opening a single visual has nothing to browse. */
  tree?: ReactNode;
  /** Data panels the owning tool contributes to the right stripe. */
  panels?: Array<IEditorPanel>;
  /** Drawn under the viewport, at whatever height it asks for. */
  footer?: ReactNode;
  /** Whether a model is on its way, reported over the viewport rather than by replacing the screen. */
  isLoading?: boolean;
  /** Why the last open failed, shown in place of a model rather than dismissing the session. */
  error?: string;
  /** Reads the failed open's source again. Absent while a tool cannot repeat its last attempt. */
  onRetry?: () => void;
  /** Reopens the picker. Absent while a tool has no way to choose a different visual. */
  onBack?: () => void;
  /** Promotes a single-model session to a browsed one. Absent while already browsing. */
  onBrowse?: () => void;
  /** Ends the selection without ending the session, which is what puts the close action in the file header. */
  onDeselect?: Nullable<() => void>;
}

/**
 * The shared preview chrome: toolbar, viewport, panel stripe and animation bar.
 */
export function VisualPreviewLayout({
  "data-testid": dataTestId = "visual-preview-layout",
  id = "visual-preview-layout",
  className,
  subtitle,
  name = null,
  tree,
  panels,
  footer,
  isLoading = false,
  error,
  onRetry,
  onBack,
  onBrowse,
  onDeselect = null,
}: IVisualPreviewLayoutProps): ReactElement {
  const viewService: VisualViewService = useInjection(VisualViewService);
  const source: IVisualRenderSource = useInjection(VISUAL_RENDER_SOURCE);

  const model: Nullable<IVisualModelViews> = source.model;
  const triangleCount: number = model ? countVisualTriangles(model, viewService.detail) : 0;
  const hasDetailLevels: boolean = (model?.levelCount ?? 1) > 1;
  const hasSkeleton: boolean = Boolean(model?.hasSkeleton);
  // A dummy pair counts: it is shaded, and comparing it flat is how a modder sees that it adds nothing.
  const hasBump: boolean = Boolean(model && source.hasBump);
  const hasAlpha: boolean = Boolean(model?.submeshes.some((submesh) => isAlphaRenderSurfaceDraw(submesh.surface)));

  const status: Array<string> = useMemo(() => {
    if (isLoading) {
      return ["Loading visual"];
    }

    return model
      ? [`${model.submeshes.length} submeshes`, `${model.vertexCount} vertices`, `${triangleCount} triangles`]
      : ["No visual open"];
  }, [isLoading, model, triangleCount]);

  useShellPanels(() => {
    const stripe: Array<IEditorPanel> = panels ? [...panels] : [];

    return tree
      ? [
          {
            icon: <AccountTreeIcon />,
            id: "project",
            isOpenByDefault: true,
            label: "Project",
            render: () => tree,
            side: "left",
          },
          ...stripe,
        ]
      : stripe;
  }, [tree, panels]);

  useShellStatus(status);

  return (
    <EditorLayout
      toolbar={
        <VisualPreviewToolbar
          hasDetailLevels={hasDetailLevels}
          hasSkeleton={hasSkeleton}
          hasBump={hasBump}
          hasAlpha={hasAlpha}
          subtitle={subtitle}
          options={viewService.options}
          lighting={viewService.lighting}
          detail={viewService.detail}
          onChangeOptions={viewService.setOptions}
          onChangeLighting={viewService.setLighting}
          onChangeDetail={viewService.setDetail}
          onBack={onBack}
          onBrowse={onBrowse}
        />
      }
      footer={footer}
    >
      <div className={"flex min-h-0 min-w-0 grow flex-col"}>
        {name ? (
          <EditorFileHeader
            data-testid={"visual-file-header"}
            name={name}
            closeLabel={"Close visual"}
            closeDescription={"Clear the selection and close this visual"}
            onClose={onDeselect ?? undefined}
          />
        ) : null}

        <div
          data-testid={dataTestId}
          id={id}
          className={cn("relative flex min-h-0 min-w-0 flex-1 overflow-hidden", className)}
        >
          <VisualPreviewViewport />

          {!model && !isLoading ? <VisualPreviewEmpty error={error} onRetry={onRetry} /> : null}

          {isLoading ? (
            <div className={"pointer-events-none absolute inset-0 flex items-center justify-center"}>
              <DelayedProgress isOnViewport={true} label={"Loading visual…"} />
            </div>
          ) : null}
        </div>
      </div>
    </EditorLayout>
  );
}
