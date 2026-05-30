# Release Checklist

## Preconditions
- Confirm the active release plan is complete or that remaining work has been moved into later plans.
- Make sure `plans/` contains only unreleased work.

## Validation
- Run `cargo test --lib` before finalizing the release.

## Versioning and Docs
- Bump the workspace version in `Cargo.toml` to match the release tag.
- Summarize the plan and add it to the release section in `README.md`.
- Update `CHANGELOG.md` without mentioning test counts.
- Update the release showcase image flow in `README.md` and `index.html`.

## Plans and Packaging
- Move completed plans to `plans/old_plans/`.
- Run `build_scripts/windows-build.ps1` to create the Windows zip under `builds/`.
- Leave `builds/` uncommitted.

## Release Commands
- Commit all release changes, tag the release, and push the tag.
- Create the GitHub release and attach the staged Windows zip.