import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import * as dialog from "@tauri-apps/plugin-dialog";
import { fireEvent, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ArchivesExplorerApplication } from "@/applications/archives-explorer/ArchivesExplorerApplication";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { AssetService } from "@/core/assets/services";
import { ArchiveProject } from "@/core/bindings/types/xrf-archive";
import { ApplicationShellFrame } from "@/core/shell/ApplicationShellFrame";
import { EditorBusyProvider } from "@/core/shell/EditorBusyContext";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { EditorPanelsProvider } from "@/core/shell/panel/context";
import {
  mockArchiveFileDescriptor,
  mockArchiveSharedPayload,
  mockArchivesProject,
  mockPathCollision,
} from "@/fixtures/mocks/archive.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

const TEXT_FILE = mockArchiveFileDescriptor({ name: "readme.ltx", sizeReal: 1024, sizeCompressed: 1024 });

const BINARY_FILE = mockArchiveFileDescriptor({
  name: "texture.dds",
  sizeReal: 2048,
  sizeCompressed: 2048,
});

const MESH_FILE = mockArchiveFileDescriptor({
  name: "actor.omf",
  sizeReal: 4096,
  sizeCompressed: 4096,
});

const PROJECT: ArchiveProject = mockArchivesProject([TEXT_FILE, BINARY_FILE, MESH_FILE]);

describe("opened archives editor", () => {
  beforeEach(() => {
    window.localStorage.clear();

    // The picture reaches the element as an object url now, so the panel needs one jsdom does not mint on its own.
    jest.spyOn(URL, "createObjectURL").mockImplementation(() => "blob:decoded-texture");
    jest.spyOn(URL, "revokeObjectURL").mockImplementation(() => undefined);

    setMockInvokeResponses({
      ["plugin:archives|get_project"]: PROJECT,
      ["plugin:archives|list_shared_payloads"]: [],
      ["plugin:archives|read_file"]: {
        name: TEXT_FILE.name,
        content: "line one\nline two",
        size: TEXT_FILE.sizeReal,
      },
      ["plugin:archives|close_project"]: undefined,
      ["plugin:archives|describe_image"]: {
        size: BINARY_FILE.sizeReal,
        shape: { width: 64, height: 64, mipmapLevels: 1, format: "DXT1" },
      },
      ["plugin:archives|read_image"]: new Uint8Array([0x89, 0x50, 0x4e, 0x47]).buffer,
    });
  });

  function renderEditor() {
    return renderWithProviders(
      <>
        <ArchivesExplorerApplication />
        <ApplicationStatusBar />
      </>,
      { route: "/archives-explorer", bindings: [AssetService, ArchivesService] }
    );
  }

  it("presents archive context, aggregate status, and a guided empty state", async () => {
    const { findByText, getByText } = renderEditor();

    expect(await findByText("Select a file to preview")).toBeInTheDocument();
    expect(getByText("Archives explorer")).toBeInTheDocument();
    expect(getByText("C:\\game\\database")).toBeInTheDocument();
    expect(getByText("1 archives")).toBeInTheDocument();
    expect(getByText("3 files")).toBeInTheDocument();
    expect(getByText("7 KB")).toBeInTheDocument();
  });

  it("selects and renders readable files as code with line numbers", async () => {
    const { findByLabelText, findByText } = renderEditor();

    await userEvent.dblClick(await findByText("readme.ltx"));

    const viewer: HTMLElement = await findByLabelText("Contents of readme.ltx");
    const [lineNumbers, contents] = Array.from(viewer.querySelectorAll("pre"));

    expect(lineNumbers).toHaveTextContent("1 2");
    expect(contents).toHaveTextContent("line one line two");
  });

  it("decodes a texture into a picture rather than refusing it", async () => {
    const { findByAltText, findByText } = renderEditor();

    await userEvent.dblClick(await findByText("texture.dds"));

    // Compressed and not a readable extension, so the text path would have refused it outright.
    expect(await findByAltText(BINARY_FILE.name)).toHaveAttribute("src", "blob:decoded-texture");
    expect(await findByText("64 x 64 · DXT1 · no mips")).toBeInTheDocument();
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", {
      path: BINARY_FILE.name,
    });
  });

  it("selects genuinely unsupported files without asking the backend to read them", async () => {
    const { findByText, getByText } = renderEditor();

    await userEvent.dblClick(await findByText("actor.omf"));

    expect(getByText("Preview unavailable")).toBeInTheDocument();
    expect(getByText(/this file type does not have a text preview/)).toBeInTheDocument();
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", {
      path: MESH_FILE.name,
    });
  });

  it("filters files and clears the filter", async () => {
    const { findByRole, findByText, getByLabelText, getByText, queryByText } = renderEditor();
    const search: HTMLElement = await findByRole("textbox", { name: "Filter archive files" });

    fireEvent.change(search, { target: { value: "readme" } });

    expect(search).toHaveValue("readme");

    await waitFor(() => expect(queryByText("texture.dds")).not.toBeInTheDocument());
    expect(getByText("readme.ltx")).toBeInTheDocument();

    await userEvent.click(getByLabelText("Clear filter"));

    expect(await findByText("texture.dds")).toBeInTheDocument();
  });

  it("expands directories without reading and restores expansion after filtering", async () => {
    const nestedFile = mockArchiveFileDescriptor({ name: "configs\\system.ltx", sizeReal: 512, sizeCompressed: 512 });

    setMockInvokeResponses({
      ["plugin:archives|get_project"]: mockArchivesProject([nestedFile, BINARY_FILE]),
    });

    const { findByLabelText, findByRole, findByText, getByLabelText, queryByText } = renderEditor();

    await userEvent.dblClick(await findByText("configs"));

    expect(await findByText("system.ltx")).toBeInTheDocument();
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());

    const search: HTMLElement = await findByRole("textbox", { name: "Filter archive files" });

    fireEvent.change(search, { target: { value: "texture" } });
    await waitFor(() => expect(queryByText("system.ltx")).not.toBeInTheDocument());

    await userEvent.click(getByLabelText("Clear filter"));
    expect(await findByText("system.ltx")).toBeInTheDocument();
    expect(await findByLabelText("Filter archive files")).toHaveValue("");
  });

  it("keeps the selection and retries a failed file read", async () => {
    let readCount: number = 0;

    setMockInvokeResponses({
      ["plugin:archives|get_project"]: PROJECT,
      ["plugin:archives|read_file"]: () => {
        readCount += 1;

        if (readCount === 1) {
          throw new Error("temporary read failure");
        }

        return { name: TEXT_FILE.name, content: "recovered", size: TEXT_FILE.sizeReal };
      },
    });

    const { findByLabelText, findByRole, findByText } = renderEditor();

    await userEvent.dblClick(await findByText("readme.ltx"));

    expect(await findByText("Could not read this file")).toBeInTheDocument();
    expect(await findByText("temporary read failure")).toBeInTheDocument();

    await userEvent.click(await findByRole("button", { name: "Retry" }));

    expect(await findByLabelText("Contents of readme.ltx")).toHaveTextContent("recovered");
    expect(readCount).toBe(2);
  });

  it("keeps file details collapsed until its tool button is used", async () => {
    const { findByLabelText, findByText, queryByText } = renderWithProviders(
      <EditorBusyProvider>
        <EditorPanelsProvider>
          <ApplicationShellFrame>
            <ArchivesExplorerApplication />
          </ApplicationShellFrame>
        </EditorPanelsProvider>
      </EditorBusyProvider>,
      { route: "/archives-explorer", bindings: [AssetService, ArchivesService] }
    );

    const detailsButton: HTMLElement = await findByLabelText("File details");

    expect(queryByText("Select a file to inspect its archive metadata.")).not.toBeInTheDocument();

    await userEvent.click(detailsButton);

    expect(await findByText("Select a file to inspect its archive metadata.")).toBeInTheDocument();
  });

  it("renders the selected file metadata in Details", async () => {
    const { findByLabelText, findByText } = renderWithProviders(
      <EditorBusyProvider>
        <EditorPanelsProvider>
          <ApplicationShellFrame>
            <ArchivesExplorerApplication />
          </ApplicationShellFrame>
        </EditorPanelsProvider>
      </EditorBusyProvider>,
      { route: "/archives-explorer", bindings: [AssetService, ArchivesService] }
    );

    await userEvent.dblClick(await findByText("texture.dds"));
    await userEvent.click(await findByLabelText("File details"));

    expect(await findByText("Source archive")).toBeInTheDocument();
    expect(await findByText("C:\\game\\database\\configs.db0")).toBeInTheDocument();
    expect(await findByText("0x12345678")).toBeInTheDocument();
    expect(await findByText("Stored")).toBeInTheDocument();
    expect(await findByText("No other entry reads these bytes")).toBeInTheDocument();
  });

  it("names the entries that read the selected file's bytes, as derived rather than recorded", async () => {
    // The format keeps no alias field, so the panel says what reads alike and labels it as derived rather than as
    // something the packer wrote down.
    setMockInvokeResponses({
      ["plugin:archives|get_project"]: PROJECT,
      ["plugin:archives|list_shared_payloads"]: [mockArchiveSharedPayload(BINARY_FILE, ["texture_copy.dds"])],
      ["plugin:archives|describe_image"]: {
        size: BINARY_FILE.sizeReal,
        shape: { width: 64, height: 64, mipmapLevels: 1, format: "DXT1" },
      },
      ["plugin:archives|read_image"]: new Uint8Array([0x89, 0x50, 0x4e, 0x47]).buffer,
    });

    const { findByLabelText, findByText } = renderWithProviders(
      <EditorBusyProvider>
        <EditorPanelsProvider>
          <ApplicationShellFrame>
            <ArchivesExplorerApplication />
          </ApplicationShellFrame>
        </EditorPanelsProvider>
      </EditorBusyProvider>,
      { route: "/archives-explorer", bindings: [AssetService, ArchivesService] }
    );

    await userEvent.dblClick(await findByText("texture.dds"));
    await userEvent.click(await findByLabelText("File details"));

    expect(await findByText(/1 other entry reads these bytes, derived from equal descriptors/)).toBeInTheDocument();
    expect(await findByText("texture_copy.dds")).toBeInTheDocument();
  });

  it("says nothing about reachability when every entry resolves to a path of its own", async () => {
    const { findByText, queryByText } = renderEditor();

    expect(await findByText("Select a file to preview")).toBeInTheDocument();
    expect(queryByText(/cannot be reached/)).not.toBeInTheDocument();
  });

  it("reports unreachable entries unprompted and names them in its panel", async () => {
    // The explorer used to show the winner and nothing else, so an entry nobody could reach was indistinguishable
    // from one nobody packed.
    setMockInvokeResponses({
      ["plugin:archives|get_project"]: PROJECT,
      ["plugin:archives|list_collisions"]: [mockPathCollision()],
    });

    const { findByLabelText, findByText } = renderWithProviders(
      <EditorBusyProvider>
        <EditorPanelsProvider>
          <ApplicationShellFrame>
            <ArchivesExplorerApplication />
          </ApplicationShellFrame>
        </EditorPanelsProvider>
      </EditorBusyProvider>,
      { route: "/archives-explorer", bindings: [AssetService, ArchivesService] }
    );

    expect(await findByText(/1 file\(s\) here cannot be reached/)).toBeInTheDocument();

    await userEvent.click(await findByLabelText("Unreachable files"));

    expect(await findByText("C:/game/database/patch.db0::Textures/A.DDS")).toBeInTheDocument();
    expect(await findByText("C:/game/database/configs.db0::textures/a.dds")).toBeInTheDocument();
  });

  it("dismisses the reachability notice without hiding the entries themselves", async () => {
    setMockInvokeResponses({
      ["plugin:archives|get_project"]: PROJECT,
      ["plugin:archives|list_collisions"]: [mockPathCollision()],
    });

    const { findByLabelText, findByText, queryByText } = renderWithProviders(
      <EditorBusyProvider>
        <EditorPanelsProvider>
          <ApplicationShellFrame>
            <ArchivesExplorerApplication />
          </ApplicationShellFrame>
        </EditorPanelsProvider>
      </EditorBusyProvider>,
      { route: "/archives-explorer", bindings: [AssetService, ArchivesService] }
    );

    await userEvent.click(await findByLabelText("Dismiss unreachable files notice"));

    await waitFor(() => expect(queryByText(/cannot be reached/)).not.toBeInTheDocument());

    await userEvent.click(await findByLabelText("Unreachable files"));

    expect(await findByText("C:/game/database/patch.db0::Textures/A.DDS")).toBeInTheDocument();
  });

  it("closes into its own picker rather than navigating away", async () => {
    // Closing used to leave for the archives landing pane. There is no pane above an application any
    // more, and the application already draws its picker whenever nothing is open.
    const { findByLabelText, findByText, queryByText } = renderWithProviders(<ArchivesExplorerApplication />, {
      route: "/archives-explorer",
      bindings: [AssetService, ArchivesService],
    });

    await userEvent.click(await findByLabelText("Back to Archives explorer"));

    expect(await findByText("Open game archives")).toBeInTheDocument();
    expect(queryByText("C:\\game\\database")).not.toBeInTheDocument();
  });

  it("stays open and reports a close failure", async () => {
    setMockInvokeResponses({
      ["plugin:archives|get_project"]: PROJECT,
      ["plugin:archives|close_project"]: () => {
        throw new Error("archive is busy");
      },
    });

    const { findByLabelText, findByText, getByText } = renderEditor();

    await userEvent.click(await findByLabelText("Back to Archives explorer"));

    expect(await findByText("Could not close archives: archive is busy")).toBeInTheDocument();
    expect(getByText("Archives explorer")).toBeInTheDocument();
  });

  it("locks navigation while a file is being written to disk", async () => {
    // Extraction writes outside the archive; leaving mid-write leaves it running against a screen
    // nobody can see. The rail is the thing that has to stop, not just the button that started it.
    const save = jest.spyOn(dialog, "save").mockResolvedValue("C:\\out\\readme.ltx");

    setMockInvokeResponses({
      ["plugin:archives|get_project"]: PROJECT,
      ["plugin:archives|read_file"]: { name: TEXT_FILE.name, content: "line", size: 4 },
      // Never settles, so the editor stays mid-extraction for the length of the assertion.
      ["plugin:archives|extract_file"]: () => new Promise(() => {}),
    });

    const { findByLabelText, findByText, getByLabelText } = renderWithProviders(
      <EditorBusyProvider>
        <EditorPanelsProvider>
          <ApplicationShellFrame>
            <ArchivesExplorerApplication />
          </ApplicationShellFrame>
        </EditorPanelsProvider>
      </EditorBusyProvider>,
      { route: "/archives-explorer", bindings: [AssetService, ArchivesService] }
    );

    await userEvent.dblClick(await findByText("readme.ltx"));
    await userEvent.click(await findByLabelText("Extract file"));

    await waitFor(() => expect(getByLabelText("Back to Archives explorer")).toBeDisabled());

    save.mockRestore();
  });

  it("supersedes a read still in flight with the next file opened", async () => {
    setMockInvokeResponses({
      ["plugin:archives|get_project"]: PROJECT,
      // Never settles, so the first selection is still in flight when the second one is made.
      ["plugin:archives|read_file"]: () => new Promise(() => {}),
      ["plugin:archives|describe_image"]: {
        size: BINARY_FILE.sizeReal,
        shape: { width: 64, height: 64, mipmapLevels: 1, format: "DXT1" },
      },
      ["plugin:archives|read_image"]: new Uint8Array([0x89, 0x50, 0x4e, 0x47]).buffer,
    });

    const { findByAltText, findByText, getByText } = renderEditor();

    await userEvent.dblClick(await findByText("readme.ltx"));

    // A texture, so it reaches the backend on its own decode command. A read cannot be waited out - a
    // large one holds every open for its whole duration - so the newer gesture abandons it instead.
    await userEvent.dblClick(getByText("texture.dds"));

    expect(await findByAltText(BINARY_FILE.name)).toHaveAttribute("src", "blob:decoded-texture");
  });
});
