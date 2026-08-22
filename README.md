# DevAtlas

DevAtlas is a desktop-first repository intelligence platform. The current implementation focuses on the MVP desktop shell under `apps/desktop/src`: repository scanning, repository explorer, semantic graph visualization, documentation generation, diagram generation, knowledge package export, and local workspace settings.

AI Chat, cloud sync, marketplace, team collaboration, security dashboards, performance dashboards, MCP runtime, and plugin runtime are outside the current MVP UI scope.

## Current MVP Scope

Included now:

- Repository scanner
- Technology detection
- Repository file explorer
- Internal knowledge graph construction
- Graph root preview on the Graph page
- Full-screen interactive graph workspace
- Documentation generator
- Diagram generator
- Knowledge package export with materialized artifacts
- Desktop React/Tauri presentation shell
- Square, yellow-gold MVP UI kit built from source-owned shadcn-style primitives

## Desktop Screens

### Dashboard

The Dashboard summarizes the current repository state, detected technologies, generated documents, generated diagrams, and generated artifact paths.

### Scanner

The Scanner page supports:

- Opening a repository path
- Browsing a repository folder in the Tauri desktop runtime
- Detecting technologies
- Analyzing files
- Selecting analysis scope by folder or file
- Surfacing scan results into dashboard, explorer, graph, documentation, diagram, and export flows

The scanner walks all source files by default and excludes dependency, build, cache, and generated-output folders such as `.git`, `node_modules`, `target`, `dist`, `build`, `out`, `coverage`, `.next`, `.turbo`, `.cache`, `.vite`, and `vendor`. The optional `maxFiles` command argument and Settings field can cap the scan for very large repositories, but the default remains uncapped for complete local code analysis.

### Explorer

The Explorer page lists repository files with a file tree, copy actions, and refresh controls.

### Graph

The Graph page intentionally shows only the repository root on the page. It does not render the full graph or expose canvas interaction in the normal page view.

The full graph is available through the `Open full graph` full-screen workspace. The interactive workspace supports:

- Pan by dragging the canvas
- Right-click drag to pan the canvas
- Mouse-wheel zoom
- Node dragging
- Node and edge hover tooltips
- Fit graph
- Reset view
- Reset node positions back to the original graph layout

This keeps the normal page lightweight while reserving complex graph interaction for a focused full-app workspace.

### Documentation

The Documentation page generates, previews, and copies generated knowledge artifacts.

Current documentation generation is treated as an MVP deterministic/template-based flow. The next semantic improvement direction is to generate documentation from a structured documentation plan based on graph topology, symbols, dependencies, entrypoints, technology clusters, and evidence-backed summaries instead of generic repository descriptions.

Planned documentation structure:

- `README.md`
- `ARCHITECTURE.md`
- `API.md`
- `DEVELOPER_GUIDE.md`
- `RISKS.md`
- `CHANGELOG_SUMMARY.md` when git history is available

Planned quality metadata:

- Coverage score
- Semantic score
- Source file count
- Symbol count
- Graph edge count
- Missing evidence sections
- Documentation warnings

### Diagrams

The Diagrams page generates, previews, and copies generated diagram artifacts.

### Exports

The Exports page selects an output folder and exports the generated knowledge package.

### Settings

The Settings page controls:

- Light or dark theme
- Right-side workspace background style
- Sidebar collapse
- Motion enablement
- Scanner workflow automation

Theme changes affect the full application shell, including the sidebar, navigation rail, status surfaces, and main workspace. These preferences are stored in local storage when available. Scanner automation settings control whether Browse immediately opens the selected folder, whether Analyze also generates documentation and diagrams, and an optional scan file limit.

## Architecture

```text
React
  -> Tauri Commands
  -> App Core
  -> Rust Engines
  -> Storage / Export Artifacts
```

Business logic lives in Rust crates. React is presentation only and calls Tauri commands through `apps/desktop/src/services/commands.ts`.

Frontend state is split by responsibility:

- Zustand stores current session and presentation state.
- TanStack Query caches backend-derived server state.
- Tailwind CSS is integrated through the Vite plugin for utility styling.
- Source-owned UI primitives live under `apps/desktop/src/components/ui`.
- ErrorBoundary protects the React presentation tree and displays a local fallback when rendering fails.

The desktop app follows a Next.js-like route folder shape inside `apps/desktop/src/app`. `app/page.tsx` is the thin entry point, `app/layout.tsx` owns the visual shell, and each desktop route has its own folder with a `page.tsx` file such as `dashboard/page.tsx`, `scanner/page.tsx`, `explorer/page.tsx`, `graphs/page.tsx`, `documentation/page.tsx`, `diagrams/page.tsx`, `exports/page.tsx`, and `settings/page.tsx`.

Shell-only pieces such as the sidebar, overlays, and route switcher live outside the route tree in `apps/desktop/src/components/app-shell`. Runtime state, command orchestration, repository actions, generated artifact actions, toast actions, and modal actions live outside the app tree in `apps/desktop/src/handlers/use-app-controller.ts`, with preferences and presentation message types colocated in `apps/desktop/src/handlers`.

## UI Design System

The desktop frontend uses a yellow-gold MVP design-token contract in `apps/desktop/src/styles/global.css`. The active UI implementation uses dark-first theming, light theme support, yellow brand accents, square controls, a `280px` sidebar width, `64px` compact collapsed sidebar behavior, Inter/system typography, and restrained page spacing.

The current desktop visual refresh uses a yellow-first minimalism direction across the React/Tauri shell. App surfaces are square-cornered, outline-free, and separated by flat neutral cards, subtle borders, and yellow inset accents. Focus and interaction feedback are expressed through yellow ring/inset accents, color shifts, and GSAP motion instead of browser outlines or rounded component chrome. The main workspace supports three backdrop modes: aurora gradient, mesh grid, and plain background.

Shared UI components use Tailwind utilities directly inside React components. Buttons, inputs, selects, textareas, switches, badges, empty states, tabs, dialogs, labels, and field rows follow this shared primitive scale so app pages inherit predictable spacing instead of ad hoc per-page sizing. The Button primitive includes GSAP press and hover feedback.

The sidebar is a source-owned React component in `apps/desktop/src/components/app-shell/sidebar.tsx`. It uses typed navigation items from `apps/desktop/src/handlers/navigation.tsx`, active-route highlighting, collapsed icon-only mode, repository status summary, and no legacy global `.sidebar` or `.nav-*` CSS selectors.

The app uses GSAP for page-stage transitions when navigating between desktop views. Page content enters with a short clip, blur, and staggered section reveal; buttons own local press and hover micro-interactions. The animation is scoped to the React presentation layer, respects reduced-motion preferences, and does not affect Tauri command execution or Rust business logic.

## Docker

Build and run the workspace containers:

```bash
docker compose up --build
```

Release validation and artifact checks:

```bash
yarn release:check
yarn release:checksums
```

CI runs `yarn validate` through GitHub Actions. Release management is documented in `docs/release-playbook.md`.

## Validation

The current frontend and Rust desktop layers are checked with:

```bash
yarn build
cargo check
```

The MVP completion gate is recorded in `docs/mvp-completion.md` and validated with `yarn mvp:check`.

Tauri release packaging may still require native Windows toolchain fixes for dependencies such as `libsqlite3-sys`, `muda`, and `tokio`.
