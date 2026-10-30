# Google Tasks Desktop

[![Build Windows](https://github.com/howard-shi55/google-tasks-desktop/actions/workflows/build-windows.yml/badge.svg?branch=main)](https://github.com/howard-shi55/google-tasks-desktop/actions/workflows/build-windows.yml)

This fork targets Windows x64 only.

## Build Windows with GitHub Actions

You can build this fork without installing Rust, Visual Studio Build Tools, or the Windows SDK on your own computer. The GitHub-hosted Windows runner provides the native build tools, and the workflow installs Rust and Bun.

1. Commit and push `.github/workflows/build-windows.yml` to the repository's default branch. The workflow must exist on the default branch before GitHub offers the manual trigger.
2. Open **Actions** in your fork. If GitHub asks you to enable workflows for the fork, enable them.
3. Select **Build Windows**, click **Run workflow**, choose the branch to build, and confirm.
4. When the run succeeds, open its summary and download **google-tasks-windows-x64** under **Artifacts**. Sign in to GitHub to download the ZIP.
5. Extract the ZIP and use the `*-setup.exe` installer, or run the application executable from the extracted `release` directory. WebView2 Runtime is required to run the application.

Artifacts expire after 14 days; rerun the workflow to produce another download. Builds use the committed dependency lockfiles and stop if dependency installation, Rust checking, or compilation fails. No additional secrets, release creation, or code-signing certificate is required. The resulting Windows binaries are unsigned.

**Build Windows** is the only build workflow and runs manually, not on push. The upstream multi-platform publishing workflow has been removed.

The Windows workflow builds the source on the selected branch. It does not itself implement a transparent background. Google sign-in, CSS injection, and desktop transparency must be tested locally after downloading the application.

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
