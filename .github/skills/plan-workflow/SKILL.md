---
name: plan-workflow
description: 'Use when updating release plans, marking progress, archiving completed plans, coordinating deprecations, or handling README/index showcase requirements.'
argument-hint: 'release version or planning task'
---

# Plan Workflow

## When to Use
- Editing `plans/v*.md`
- Splitting unfinished work into later releases
- Archiving completed plan files
- Preparing release showcase updates in `README.md` and `index.html`

## Procedure
1. Start from the active plan for the target release and keep progress plus implementation notes current while you work.
2. Keep `plans/` limited to unreleased work. Move completed plan files to `plans/old_plans/` immediately, and prefix archived sub-plans with the release version.
3. If a feature is marked for deprecation, create or update the later plan that finishes the removal instead of leaving the work implied.
4. Treat the per-release fractal as mandatory. Since v0.2.0, also maintain the showcase image flow:
   - New image in `img_resources/showcase/`
   - Previous showcase moved to `img_resources/gallery_expanded/`
   - `README.md` and `index.html` updated together
5. Keep `index.html` synchronized with the current release showcase metadata: `og:image`, `twitter:image`, showcase `<img>`, alt text, and caption.
6. Do not add colormap deliverables to release plans; colormap work belongs in `scala-chromatica`.
7. Before running the final release checklist, verify the plan is actually complete or the remaining work has been moved forward deliberately.