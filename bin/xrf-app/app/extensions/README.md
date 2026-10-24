# Webview extensions

Debug builds of `xrf-app` load unpacked browser extensions from `unpacked/` in this directory. The
directory is ignored by Git: every developer picks the extensions they debug with, and none of them
are shared through the repository.

## Usage

Create `unpacked/` and give every extension its own folder inside it, each holding the extension's
`manifest.json` at its top level:

```
extensions/unpacked/react-devtools/manifest.json
extensions/unpacked/wirestate-devtools/manifest.json
```

Copy a built extension in, or point at one that is built elsewhere with a directory junction, which
needs no elevation:

```
mklink /J extensions\unpacked\wirestate-devtools C:\path\to\wirestate\extension\dist
```

Restart the application to pick up a change. WebView2 installs an extension into its profile and
keeps it there, so an extension whose folder is later removed stays installed until it is removed
from the profile through the browser's own extensions page.

Set `XRF_APP_EXTENSIONS_DIR` to load from somewhere else instead. It replaces `unpacked/` and is
read the same way, so it must be a directory that contains one folder per extension.

## Constraints

- Windows only, and debug builds only. Chrome extensions are a WebView2 capability, and release
  builds never enable them.
- Requires WebView2 Runtime `120.0.2210.55` or newer. Older runtimes ignore extensions silently.
- Nothing but extension folders may sit in `unpacked/`. WebView2 rejects any other entry, and that
  failure aborts window creation, so the application refuses to load any extension while an invalid
  entry is present and logs which one it was. This is why the documentation lives one level up
  rather than beside the extensions.
- Extensions whose folder name starts with `_` are reserved by WebView2 and always rejected.

## Wirestate DevTools

The panel reads the hook that `DevToolsPlugin` installs, which `ApplicationProvider` registers in
development builds. Whether a DevTools panel extension surfaces inside the WebView2 DevTools window
is a property of the runtime rather than of this application.
