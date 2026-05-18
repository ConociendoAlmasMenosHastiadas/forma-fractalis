---
name: release-workflow
description: 'Use when finalizing a release, bumping the version, updating README and CHANGELOG, building the Windows package, or creating the GitHub release.'
argument-hint: 'release version'
---

# Release Workflow

## When to Use
- Preparing a tagged release
- Bumping the version number
- Updating release notes and packaging artifacts

## Guardrails
- Only run the final release workflow when the relevant plan is complete or its remaining work has been moved forward deliberately.
- Ask whether the project is at the end stage before executing the full release checklist.

## Procedure
1. Review the [release checklist](./references/checklist.md) before making release edits.
2. Run the required validation before touching versioning or packaging.
3. Update release-facing docs and showcase assets in one pass so `README.md` and `index.html` stay aligned.
4. Archive the completed plans immediately after the release is finalized.
5. Keep `builds/` as local staging only; do not commit release zip files.