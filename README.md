# Google Tasks Desktop

[![Build Windows](https://github.com/howard-shi55/google-tasks-desktop/actions/workflows/build-windows.yml/badge.svg?branch=main)](https://github.com/howard-shi55/google-tasks-desktop/actions/workflows/build-windows.yml)

This fork targets Windows x64 only.

Current version: **0.1.8**.

Version 0.1.8 disables WebView browser menus/shortcuts and restores the last normal window position and size.

Version 0.1.7 promotes common tray actions to the top level, adds an optional **Start at login** checkbox, and hides the app's Windows taskbar button.

Version 0.1.6 fixes theme and dragging injection after Google navigates from `/embed/` to `/embed/list/<id>`. Both the native URL guard and injected script accept these embed subroutes while excluding other origins and non-embed pages. It also fixes the drag permission: Tauri 2.0's Windows fetch IPC checks the origin without the page path, so the remote capability matches `https://tasks.google.com` rather than `/embed/*`. Only the start-dragging permission is granted to that origin; the script installs its handler only on embed pages.

## Build Windows with GitHub Actions

You can build this fork without installing Rust, Visual Studio Build Tools, or the Windows SDK on your own computer. The GitHub-hosted Windows runner provides the native build tools, and the workflow installs Rust and Bun.

1. Commit and push `.github/workflows/build-windows.yml` to the repository's default branch. The workflow must exist on the default branch before GitHub offers the manual trigger.
2. Open **Actions** in your fork. If GitHub asks you to enable workflows for the fork, enable them.
3. Select **Build Windows**, click **Run workflow**, choose the branch to build, and confirm.
4. When the run succeeds, open its summary and download **google-tasks-windows-x64** under **Artifacts**. Sign in to GitHub to download the ZIP.
5. Extract the ZIP and use the `*-setup.exe` installer, or run the application executable from the extracted `release` directory. WebView2 Runtime is required to run the application.

Artifacts expire after 14 days; rerun the workflow to produce another download. Builds use the committed dependency lockfiles and stop if dependency installation, Rust checking, or compilation fails. No additional secrets, release creation, or code-signing certificate is required. The resulting Windows binaries are unsigned.

**Build Windows** is the only build workflow and runs manually, not on push. The upstream multi-platform publishing workflow has been removed.

## Transparent Windows theme

The window/WebView background is transparent and frameless, without a native title bar, border, or in-window menu bar. After the Google Tasks embed page finishes loading, the app injects fixed white text with a subtle black shadow and transparent page surfaces. Menus, dialogs, listboxes, and tooltips keep a dark background for readability. Google sign-in pages are not themed. Transparency does not enable click-through.

Drag empty space in the **Tasks / My Tasks header strip** to move the window. The list dropdown and other interactive controls keep their normal clicks. The current layout uses the top 80 CSS pixels for this strip; if Google changes the header height, adjust the bound in `src-tauri/src/theme.js`. This is independent of the current list's name or language. Google sign-in pages do not get the drag handler; **Center** in the tray menu can reposition the window there.

The app does not show a Windows taskbar button. Its Google Tasks icon stays in the system tray (possibly in the hidden-icons overflow). Left-click it to show/restore the window. Right-click it for:

- **Top level:** Show / Restore, Hide, Center, Start at login, and Quit.
- **Edit:** Undo, Redo, Cut, Copy, Paste, Select All. Select/focus the task input before opening the tray menu; the app restores WebView focus and sends the corresponding native Ctrl shortcut. Shortcuts are not sent if another app is foreground or a modifier key is held. Their result depends on the focused Google editor/selection, as with normal keyboard shortcuts.
- **Window:** Minimize, Toggle Full Screen.
- **More:** About, Sign Out. Sign Out opens Google's logout flow without forcibly terminating it.

Closing the window, including Alt+F4, hides it in the tray. **Quit** exits the app. Window dragging is the only native permission granted to the remote Google Tasks origin; tray and startup actions stay in Rust.

### Browser controls and remembered window bounds

The native WebView has default browser context menus, DevTools, browser accelerators, and user zoom controls disabled. This covers browser commands such as refresh, print, save, browser navigation, and DevTools shortcuts. Function keys F1-F24 are intercepted inside the WebView, including F5 and F12; **Alt+F4** retains close-to-tray behavior. Text editing and navigation keys such as Ctrl+C/V/X/A/Z, Tab, Enter, and arrow keys remain available. Website-owned dropdowns/menus and the native tray menu continue to work; this does not block Windows-wide shortcuts.

Window geometry is stored per user at `HKCU\Software\GoogleTasksDesktop\WindowPlacementV1`. It is saved before hiding/quitting and on focus loss, without writing on every movement/resize pixel. Startup restores the last **normal** window rectangle, rather than restarting minimized, maximized, or fullscreen. Before entering fullscreen, the previous normal bounds are saved. Windows' native placement API adjusts completely offscreen positions after display changes; invalid settings fall back to the default window. Raw native bounds do not promise an identical logical size after a DPI change.

### Start at login

Check **Start at login** to launch Google Tasks automatically when your Windows user signs in; uncheck it to stop. It is opt-in and uses only the current user's `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\GoogleTasksDesktop` value, without administrator rights. Opening the app only reads the setting; it does not enable startup. The checkbox is initialized from the saved registration. Failed changes restore the saved state and mark the item as **update failed**.

Enabling startup stores the fully quoted path of the executable currently running, so install the app before enabling it. If using the portable executable, keep it in a fixed location and toggle startup off/on after moving it. The app opens normally at login. Uninstalling removes its startup value; explicit updater-mode reinstalls preserve it. If you choose to uninstall the old app before a manual reinstall, enable startup again afterward.

The injected style is reapplied after full navigation and covers new elements within the same document. It does not cross iframe or shadow-root boundaries. Because Google controls the remote page, its CSP or markup changes may affect the result. Clearing page background colors can also affect state indicators; keyboard focus and ARIA-selected/checked controls receive explicit white outlines.

Run `npm test` to check origin isolation, root/list embed routes, repeat injection, and header dragging (including excluded controls and synthetic mouse events) without a Google account. Run `cargo test --locked --lib` inside `src-tauri` to test the actual Rust navigation guard, IPC-origin ACL matching, startup registration, function-key policy, and saved bounds. Registry tests use isolated temporary subtrees, not the real Run/settings keys; they cover Unicode/quoted startup paths, persistence, idempotent disabling, unrelated-value preservation, negative monitor coordinates, invalid geometry and corrupt settings. The cloud workflow runs these checks before release compilation. These checks do not verify native WebView menus/accelerators, actual placement after restart, sign-in startup execution, or remote CSP.

After downloading, verify white text/shadow, transparency, the absent title/menu bars, header dragging, dropdown clicks, and every tray action. Verify right-clicking the WebView does not show browser menus and F5/F12/Ctrl+R/Ctrl+P/Ctrl+Shift+I do not trigger browser commands. Move/resize the window, quit, restart, and compare its position/size; repeat while hidden/minimized/fullscreen. Test text editing and Ctrl+C/V/Z, closing/restoring, list/task interactions, sign-out/in, and Quit. For startup, enable the checkbox, restart the app to check persistence, then sign out/in to check launching; disable it afterward if unwanted. These native/visual behaviors require local checking; compilation alone cannot confirm them.

## Demo Screenshot
<img width="1512" alt="Screen Shot 2023-05-31 at 9 58 13 PM" src="https://github.com/shouryan01/Google-Tasks-Desktop/assets/32345320/bee79572-7080-4730-aaa0-fb9874b98fba">

I am a heavy user of Google Tasks. I love the easy integration between all the Google services such as Gmail, Calendar and Keep. The mobile app is also excellent. However, Google forces you to use Google Tasks in this terrible interface:

<img width="1624" alt="Screen Shot 2023-06-03 at 6 50 56 PM" src="https://github.com/shouryan01/Google-Tasks-Desktop/assets/32345320/74de30d7-fb29-46cc-bb9b-7116d5cd9eaa">

## Resource Use

The following comparisons are historical upstream macOS measurements, not benchmarks of this Windows fork.

This is no comparison. An optimized-native M1 Mac app vs Google Chrome? Not fair. To make it more fair, Google Chrome is an unused browser (I use Firefox!). EVEN then, Google Tasks running as a sidebar app in Gmail uses 10x more memory and 6x more CPU than Google Tasks Desktop.

### CPU Comparison

<img width="1072" alt="Screen Shot 2023-06-03 at 6 46 34 PM" src="https://github.com/shouryan01/Google-Tasks-Desktop/assets/32345320/1267c166-6cec-42a0-a3aa-9f1356a20b6a">

<img width="1072" alt="Screen Shot 2023-06-03 at 6 46 28 PM" src="https://github.com/shouryan01/Google-Tasks-Desktop/assets/32345320/1a054471-0624-49da-aabb-d3109c3362c8">

### RAM Comparison

<img width="1072" alt="Screen Shot 2023-06-03 at 6 45 45 PM" src="https://github.com/shouryan01/Google-Tasks-Desktop/assets/32345320/94de9ad1-a0f2-43d0-a0f8-91629ea32d59">

<img width="1072" alt="Screen Shot 2023-06-03 at 6 45 55 PM" src="https://github.com/shouryan01/Google-Tasks-Desktop/assets/32345320/c21736ae-c410-46c7-92c5-c99f5c531c3d">

That's it! The app is a no-nonsense wrapper around Google Tasks, it just works! This implementation meets all my needs and I am satisfied with it. 
If you find it useful and would want more features, please raise a PR! Issues work too, but I'll be resolving those more slowly.
