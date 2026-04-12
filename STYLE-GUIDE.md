# Chuck — Unified Design System & Style Guide

> **"Warm Ink"** — warm paper/ink neutrals with a copper accent.
> An editorial, tactile personality that avoids generic "AI slop" aesthetics.

This is the **canonical design reference** for all UI components in Chuck. Every panel,
form, card, button, and interactive element must use the shared classes and tokens documented
here. When this guide conflicts with scoped component styles, the guide wins.

**Source of truth:** `src/app.css` — all design tokens + 22 unified component categories.

---

## Table of Contents

1. [Design Tokens](#1-design-tokens)
2. [Typography](#2-typography)
3. [Color System](#3-color-system)
4. [Spacing & Layout](#4-spacing--layout)
5. [Elevation & Motion](#5-elevation--motion)
6. [Panel Layout](#6-panel-layout)
7. [Section Headings](#7-section-headings)
8. [Cards](#8-cards)
9. [Expandable Detail](#9-expandable-detail)
10. [Buttons](#10-buttons)
11. [Badges](#11-badges)
12. [Status Indicators](#12-status-indicators)
13. [Toggle Switch](#13-toggle-switch)
14. [Form Controls](#14-form-controls)
15. [Setting Row](#15-setting-row)
16. [Search Input](#16-search-input)
17. [Empty State](#17-empty-state)
18. [Banners](#18-banners)
19. [Code Snippet](#19-code-snippet)
20. [Segmented Control & Tab Bar](#20-segmented-control--tab-bar)
21. [Spinner & Progress](#21-spinner--progress)
22. [Checkbox List](#22-checkbox-list)
23. [File Item](#23-file-item)
24. [Form Actions](#24-form-actions)
25. [Kbd](#25-kbd)
26. [Panel-Specific Scoped Styles](#26-panel-specific-scoped-styles)
27. [Do's and Don'ts](#27-dos-and-donts)
28. [Agents Panel Reference](#28-agents-panel-reference)
29. [Skills Panel Reference](#29-skills-panel-reference)
30. [MCP Panel Reference](#30-mcp-panel-reference)
31. [Settings Panel Reference](#31-settings-panel-reference)
32. [Projects Panel Reference](#32-projects-panel-reference)

---

## 1. Design Tokens

All values are CSS custom properties defined on `:root` in `src/app.css`. Dark mode overrides
are on `[data-theme="dark"]`. System dark mode mirrors dark via `@media (prefers-color-scheme: dark)`.

**Rule:** Never hardcode colors, font sizes, spacing, or radii. Always use `var(--token-name)`.

### Token Categories

| Category   | Prefix                                         | Example                                                                   |
| ---------- | ---------------------------------------------- | ------------------------------------------------------------------------- |
| Surfaces   | `--color-bg-*`                                 | `--color-bg-primary`, `--color-bg-secondary`, `--color-bg-hover`          |
| Text       | `--color-text-*`                               | `--color-text-primary`, `--color-text-secondary`, `--color-text-tertiary` |
| Borders    | `--color-border-*`                             | `--color-border-primary`, `--color-border-focus`                          |
| Accent     | `--color-accent-*`                             | `--color-accent-copper`, `--color-accent-copper-hover`                    |
| Semantic   | `--color-success/warning/error`                | `--color-success`, `--color-error-subtle`                                 |
| Shadows    | `--shadow-*`                                   | `--shadow-sm`, `--shadow-md`, `--shadow-lg`, `--shadow-input-focus`       |
| Radii      | `--radius-*`                                   | `--radius-sm` (6px), `--radius-md` (10px), `--radius-lg` (16px)           |
| Spacing    | `--spacing-*`                                  | `--spacing-xs` (4px) through `--spacing-4xl` (64px)                       |
| Typography | `--font-*`, `--font-size-*`, `--font-weight-*` | See Typography section                                                    |
| Motion     | `--transition-*`                               | `--transition-fast` (120ms), `--transition-spring` (400ms)                |

---

## 2. Typography

### Font Stacks

| Token            | Font                       | Usage                                                              |
| ---------------- | -------------------------- | ------------------------------------------------------------------ |
| `--font-display` | Instrument Serif           | Panel titles, welcome headings, editorial text. Always **italic**. |
| `--font-sans`    | Plus Jakarta Sans Variable | All body text, labels, descriptions, buttons                       |
| `--font-mono`    | JetBrains Mono             | Code blocks, device codes, monospaced inputs                       |

### Size Scale

| Token              | Size   | Usage                                                   |
| ------------------ | ------ | ------------------------------------------------------- |
| `--font-size-2xs`  | 10px   | Badges, smallest labels                                 |
| `--font-size-xs`   | 11px   | Form labels, hints, section headings, card descriptions |
| `--font-size-sm`   | 13px   | Body text, card titles, form inputs, buttons            |
| `--font-size-base` | 14.5px | Default body (root-level)                               |
| `--font-size-md`   | 16px   | Empty state titles, medium headings                     |
| `--font-size-lg`   | 20px   | Card icons, large display elements                      |
| `--font-size-xl`   | 28px   | Panel titles (`.panel-title`)                           |
| `--font-size-2xl`  | 40px   | Hero/welcome headings only                              |

### Weight Scale

| Token                    | Weight | Usage                                 |
| ------------------------ | ------ | ------------------------------------- |
| `--font-weight-normal`   | 400    | Body text, descriptions               |
| `--font-weight-medium`   | 500    | Form labels, card titles, active tabs |
| `--font-weight-semibold` | 600    | Section headings, emphasis            |
| `--font-weight-bold`     | 700    | Reserved (rarely used)                |

---

## 3. Color System

### Light Theme (`:root`)

| Role           | Token                    | Value                  | Usage                                       |
| -------------- | ------------------------ | ---------------------- | ------------------------------------------- |
| Primary BG     | `--color-bg-primary`     | `#faf9f7`              | Page background, input background fallback  |
| Secondary BG   | `--color-bg-secondary`   | `#f3f1ed`              | Cards, sidebar                              |
| Tertiary BG    | `--color-bg-tertiary`    | `#eae7e1`              | Toggle tracks, code blocks                  |
| Hover          | `--color-bg-hover`       | `rgba(28,25,23, 0.04)` | Hover states                                |
| Active         | `--color-bg-active`      | `rgba(28,25,23, 0.08)` | Active/pressed states                       |
| Primary Text   | `--color-text-primary`   | `#1c1917`              | Headings, card titles, input text           |
| Secondary Text | `--color-text-secondary` | `#78716c`              | Descriptions, form labels                   |
| Tertiary Text  | `--color-text-tertiary`  | `#a8a29e`              | Hints, placeholders, muted text             |
| Copper Accent  | `--color-accent-copper`  | `#b45309`              | Focus rings, accent buttons, links, toggles |
| Primary Border | `--color-border-primary` | `#e7e5e4`              | Card borders, dividers                      |
| Focus Border   | `--color-border-focus`   | `#b45309`              | Input focus, card hover                     |

### Dark Theme (`[data-theme="dark"]`)

| Role           | Token                    | Value     |
| -------------- | ------------------------ | --------- |
| Primary BG     | `--color-bg-primary`     | `#1e1c1a` |
| Secondary BG   | `--color-bg-secondary`   | `#262422` |
| Tertiary BG    | `--color-bg-tertiary`    | `#302e2b` |
| Primary Text   | `--color-text-primary`   | `#edebe9` |
| Copper Accent  | `--color-accent-copper`  | `#e8a030` |
| Primary Border | `--color-border-primary` | `#38352f` |

### Semantic Colors

| Token             | Light     | Dark      | Usage                                       |
| ----------------- | --------- | --------- | ------------------------------------------- |
| `--color-success` | `#166534` | `#4ade80` | Connected status, success banners           |
| `--color-warning` | `#92400e` | `#fbbf24` | Warning banners, caution states             |
| `--color-error`   | `#b91c1c` | `#f87171` | Error banners, danger buttons, disconnected |

### Semi-Transparent Backgrounds

Use `color-mix(in srgb, ...)` for semantic backgrounds:

```css
/* Badge background */
background: color-mix(in srgb, var(--color-accent-copper) 12%, transparent);

/* Error background */
background: color-mix(in srgb, var(--color-error) 8%, transparent);

/* Danger button hover */
background: color-mix(in srgb, var(--color-error) 8%, transparent);
```

> **Note:** Requires WebKit 15.4+ (all Tauri-supported platforms meet this).

---

## 4. Spacing & Layout

### Spacing Scale

| Token           | Value | Common Usage                             |
| --------------- | ----- | ---------------------------------------- |
| `--spacing-xs`  | 4px   | Inner gaps (badge padding, icon spacing) |
| `--spacing-sm`  | 8px   | Card padding, form gaps, button padding  |
| `--spacing-md`  | 12px  | Panel header padding, card body          |
| `--spacing-lg`  | 16px  | Panel body padding, section gaps         |
| `--spacing-xl`  | 24px  | Large section gaps                       |
| `--spacing-2xl` | 32px  | Search icon offset                       |
| `--spacing-3xl` | 48px  | Empty state padding                      |

### Layout Constants

| Token                       | Value | Purpose                     |
| --------------------------- | ----- | --------------------------- |
| `--sidebar-width`           | 260px | Sidebar expanded width      |
| `--sidebar-collapsed-width` | 52px  | Sidebar icon-only width     |
| `--titlebar-height`         | 38px  | Custom title bar height     |
| `--content-max-width`       | 680px | Max width for content areas |

### Radius Scale

| Token           | Value  | Usage                                  |
| --------------- | ------ | -------------------------------------- |
| `--radius-sm`   | 6px    | Inputs, buttons, badges, cards (inner) |
| `--radius-md`   | 10px   | Cards, banners, code blocks            |
| `--radius-lg`   | 16px   | Large containers, modals               |
| `--radius-xl`   | 24px   | Reserved                               |
| `--radius-full` | 9999px | Pills, toggles, tab pills              |

---

## 5. Elevation & Motion

### Shadows

| Token                  | Usage                             |
| ---------------------- | --------------------------------- |
| `--shadow-sm`          | Card hover lift, active tab pills |
| `--shadow-md`          | Dropdowns, popovers               |
| `--shadow-lg`          | Modals, floating panels           |
| `--shadow-input`       | Default input shadow              |
| `--shadow-input-focus` | Input focus ring (copper glow)    |

### Transitions

| Token                 | Duration                                | Usage                                  |
| --------------------- | --------------------------------------- | -------------------------------------- |
| `--transition-fast`   | 120ms ease                              | Hover states, border color, background |
| `--transition-normal` | 200ms ease                              | Toggle track, progress bar             |
| `--transition-slow`   | 320ms ease                              | Panel transitions, slide-overs         |
| `--transition-spring` | 400ms cubic-bezier(0.34, 1.56, 0.64, 1) | Sidebar, playful micro-interactions    |

### Animations

| Name       | Effect                       | Usage                        |
| ---------- | ---------------------------- | ---------------------------- |
| `fadeIn`   | Opacity 0→1                  | Expandable details, overlays |
| `fadeInUp` | Opacity + translateY(12px→0) | Page entries, list items     |
| `scaleIn`  | Opacity + scale(0.96→1)      | Modals, dropdown menus       |
| `spin`     | rotate(0→360deg)             | Spinners                     |

---

## 6. Panel Layout

Every panel page (Agents, Skills, MCP, Settings, Projects) uses the same outer structure.

### Classes

| Class                | Element    | Purpose                                                                                                                       |
| -------------------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `.panel`             | `<div>`    | Flex column container, full height, hidden overflow                                                                           |
| `.panel-header`      | `<header>` | Top bar with back button + title. `border-bottom`, flex row, `data-tauri-drag-region`                                         |
| `.panel-back`        | `<button>` | SVG chevron icon button in copper accent (`--color-accent-copper`). Hover: `--color-accent-copper-hover` + `--color-bg-hover` |
| `.panel-title`       | `<h2>`     | Instrument Serif italic, `--font-size-xl`                                                                                     |
| `.panel-body`        | `<div>`    | Scrollable content area, `padding: --spacing-lg`                                                                              |
| `.panel-body-narrow` | `<div>`    | Same as `.panel-body` but `max-width: 640px`, centered                                                                        |

### Canonical Header Markup

**All panels must use this exact structure.** No text labels in the back button — SVG only.

```svelte
<div class="panel">
  <header class="panel-header" data-tauri-drag-region>
    <button class="panel-back" onclick={goBack} aria-label="Go back">
      <svg width="18" height="18" viewBox="0 0 16 16" fill="none">
        <path
          d="M10 3L5 8l5 5"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <h2 class="panel-title">Agents</h2>
  </header>
  <div class="panel-body">
    <!-- scrollable content -->
  </div>
</div>
```

**Header rules:**

- Always `<header>` (semantic), never `<div>`
- Always `data-tauri-drag-region` for window dragging
- Back button: SVG chevron only, no text label, `aria-label="Go back"`
- Title: always `<h2>`, never `<h1>`
- Optional action button (e.g., "+ New Agent") after the title

### When to use `.panel-body` vs `.panel-body-narrow`

- **`.panel-body`** — List views (agent cards, MCP servers, skill lists)
- **`.panel-body-narrow`** — Form views (create/edit agent, MCP server form, settings)

---

## 7. Section Headings

### Class: `.section-heading`

Uppercase, letter-spaced, tertiary color. Used to group content within a panel body.

```css
font-size: var(--font-size-xs); /* 11px */
font-weight: var(--font-weight-semibold);
text-transform: uppercase;
letter-spacing: 0.06em;
color: var(--color-text-tertiary);
margin: var(--spacing-lg) 0 var(--spacing-sm) 0;
```

First `.section-heading` in a container has `margin-top: 0`.

```svelte
<h3 class="section-heading">Built-in Skills</h3>
```

---

## 8. Cards

The primary content container. Used for agents, skills, MCP servers, project entries, and registry results.

### Base Card

| Class           | Purpose                                                                                |
| --------------- | -------------------------------------------------------------------------------------- |
| `.card`         | Base: secondary bg, primary border, `--radius-md`, hover changes border to focus color |
| `.card + .card` | Auto margin-top: `--spacing-sm` between siblings                                       |

### Variants

| Modifier           | Effect                                                   | When to Use                                             |
| ------------------ | -------------------------------------------------------- | ------------------------------------------------------- |
| `.card--featured`  | 3px copper left border                                   | Default/built-in items (default agent, built-in skills) |
| `.card--clickable` | Cursor pointer, hover lift (`translateY(-1px)` + shadow) | Expandable cards, selectable items                      |
| `.card--flat`      | No hover border change, default cursor                   | Static display cards (read-only info)                   |

### Card Anatomy

| Class           | Purpose                                              |
| --------------- | ---------------------------------------------------- |
| `.card-header`  | Flex row: icon + title + actions. `min-height: 24px` |
| `.card-icon`    | Emoji or icon. `--font-size-lg`, flex-shrink 0       |
| `.card-title`   | `--font-size-sm`, semibold, ellipsis overflow        |
| `.card-actions` | Right-aligned action buttons (Edit, Delete, etc.)    |
| `.card-desc`    | `--font-size-xs`, secondary color, 2-line clamp      |
| `.card-meta`    | Flex wrap row of badges below description            |

### Example

```svelte
<div class="card card--clickable">
  <div class="card-header">
    <span class="card-icon">🔬</span>
    <span class="card-title">Research Agent</span>
    <div class="card-actions">
      <button class="btn btn--ghost btn--sm">Edit</button>
      <button class="btn btn--ghost btn--sm btn--danger">Delete</button>
    </div>
  </div>
  <p class="card-desc">Deep research with web search and citations.</p>
  <div class="card-meta">
    <span class="badge badge--copper">2 skills</span>
    <span class="badge badge--neutral">aitmpl.com</span>
  </div>
</div>
```

---

## 9. Expandable Detail

Shown inside a card when the user clicks to expand. Used in Agents (agent details), Skills (instructions), and MCP (server details).

| Class                  | Purpose                                                        |
| ---------------------- | -------------------------------------------------------------- |
| `.card-detail`         | Container with secondary bg, subtle border, `fadeIn` animation |
| `.detail-section`      | Padded row with bottom border (last child: no border)          |
| `.detail-label`        | `--font-size-xs`, medium weight, secondary color               |
| `.detail-value`        | `--font-size-sm`, primary color                                |
| `.detail-value--muted` | Tertiary color, italic (for empty/placeholder values)          |

```svelte
<div class="card-detail">
  <div class="detail-section">
    <div class="detail-label">System Prompt</div>
    <div class="detail-value">You are a research assistant...</div>
  </div>
  <div class="detail-section">
    <div class="detail-label">Source</div>
    <div class="detail-value--muted">Local — not imported</div>
  </div>
</div>
```

---

## 10. Buttons

All interactive buttons use the `.btn` base class plus modifiers.

### Variants

| Class           | Appearance                         | Usage                            |
| --------------- | ---------------------------------- | -------------------------------- |
| `.btn`          | Ghost with border, secondary text  | Default/neutral actions          |
| `.btn--primary` | Dark bg (ink), inverse text        | Primary CTA (Save, Create)       |
| `.btn--accent`  | Copper bg, white text              | High-emphasis accent actions     |
| `.btn--ghost`   | No border, no bg                   | Subtle actions (toolbar buttons) |
| `.btn--link`    | No border, copper text             | Inline links styled as buttons   |
| `.btn--danger`  | Red text, red border (30% opacity) | Destructive actions (Delete)     |

### Size & Shape Modifiers

| Class        | Effect                                                 |
| ------------ | ------------------------------------------------------ |
| `.btn--pill` | Full border-radius (pill shape)                        |
| `.btn--sm`   | Smaller font + padding (`--font-size-2xs`)             |
| `.btn--icon` | Square 28×28, no border, tertiary color, centered icon |

### States

- **Hover:** Background/color shift (variant-specific)
- **Disabled:** `opacity: 0.5`, `cursor: not-allowed`
- **Focus-visible:** Uses global `outline: 2px solid --color-border-focus` (from reset)

### Combining Modifiers

```svelte
<button class="btn btn--primary">Save Agent</button>
<button class="btn btn--ghost btn--sm">Edit</button>
<button class="btn btn--danger btn--sm">Delete</button>
<button class="btn btn--accent btn--pill">+ Install</button>
<button class="btn btn--icon" aria-label="Close">
  <svg>...</svg>
</button>
```

---

## 11. Badges

Inline metadata labels. Always small, never interactive.

| Class             | Appearance                             | Usage                                |
| ----------------- | -------------------------------------- | ------------------------------------ |
| `.badge`          | Base: `--font-size-2xs`, `--radius-sm` | —                                    |
| `.badge--neutral` | Tertiary text on 10% tertiary bg       | Source type, category labels         |
| `.badge--copper`  | Copper text on 12% copper bg           | Counts ("2 skills"), accent metadata |
| `.badge--success` | Green text on 10% green bg             | "Connected", active status           |
| `.badge--error`   | Red text on 10% red bg                 | "Disconnected", error status         |
| `.badge--mono`    | Mono font, secondary text, tertiary bg | Technical labels (transport type)    |

```svelte
<span class="badge badge--copper">3 tools</span>
<span class="badge badge--mono">stdio</span>
<span class="badge badge--success">connected</span>
```

---

## 12. Status Indicators

Used for MCP server connection state and similar binary statuses.

| Class                   | Purpose                     |
| ----------------------- | --------------------------- |
| `.status`               | Flex row container with gap |
| `.status-dot`           | 6×6px circle indicator      |
| `.status--connected`    | Green dot + green text      |
| `.status--disconnected` | Red dot + red text          |
| `.status--warning`      | Amber dot + amber text      |

```svelte
<span class="status status--connected">
  <span class="status-dot"></span>
  Connected
</span>
```

---

## 13. Toggle Switch

Unified 36×20px toggle with copper accent and spring easing.

### Structure

```svelte
<label class="toggle">
  <input type="checkbox" checked={value} onchange={handler} />
  <span class="toggle-track"></span>
</label>
```

### Specs

- Track: 36×20px, `--color-bg-tertiary`, 1px border
- Thumb: 14×14px circle, `--color-text-tertiary` → `--color-accent-copper` when checked
- Checked track: 20% copper background, copper border
- Animation: `cubic-bezier(0.34, 1.56, 0.64, 1)` (spring bounce)
- Focus: `--shadow-input-focus` on focus-visible

---

## 14. Form Controls

### Field Container

```svelte
<div class="form-field">
  <label class="form-label">Name</label>
  <input class="form-input" />
  <span class="form-hint">Short description of this field</span>
</div>
```

> **Important:** Do NOT use `.form-field + .form-field` margin for spacing. Use `gap` on the parent
> container instead (e.g., `display: flex; flex-direction: column; gap: var(--spacing-lg)`).

### Classes

| Class                 | Purpose                                          |
| --------------------- | ------------------------------------------------ |
| `.form-field`         | Flex column with `gap: --spacing-xs`             |
| `.form-label`         | `--font-size-xs`, medium weight, secondary color |
| `.form-hint`          | `--font-size-xs`, tertiary color, italic         |
| `.form-input`         | Full-width text input with copper focus ring     |
| `textarea.form-input` | Min 80px height, vertical resize                 |
| `.form-input--mono`   | Monospace variant (code editors, JSON)           |
| `.form-select`        | Styled `<select>` with hover/focus states        |
| `.form-radio-group`   | Flex row of `<label>` + `<input type="radio">`   |
| `.form-error`         | Red text on 8% red bg, small rounded container   |

### Focus State

All form inputs share the same focus treatment:

```css
border-color: var(--color-border-focus); /* copper */
box-shadow: var(--shadow-input-focus); /* copper glow ring */
```

---

## 15. Setting Row

Used in Settings panel for key-value preference rows (label + control).

| Class            | Purpose                                       |
| ---------------- | --------------------------------------------- |
| `.setting-row`   | Flex row, `space-between`, vertical centering |
| `.setting-info`  | Left column: label + description              |
| `.setting-label` | `--font-size-sm`, medium weight               |
| `.setting-desc`  | `--font-size-xs`, tertiary color              |

```svelte
<div class="setting-row">
  <div class="setting-info">
    <span class="setting-label">Theme</span>
    <span class="setting-desc">Choose light, dark, or system preference</span>
  </div>
  <select class="form-select">...</select>
</div>
```

---

## 16. Search Input

Search field with icon overlay and optional loading spinner.

| Class                       | Purpose                                      |
| --------------------------- | -------------------------------------------- |
| `.search-field`             | `position: relative` wrapper                 |
| `.search-field .form-input` | Extra left + right padding for icon/spinner  |
| `.search-field-icon`        | Absolute-positioned search icon (left)       |
| `.search-spinner`           | Absolute-positioned spinner (right), 14×14px |

```svelte
<div class="search-field">
  <span class="search-field-icon"><svg>🔍</svg></span>
  <input class="form-input" placeholder="Search skills..." />
  {#if loading}<span class="search-spinner"></span>{/if}
</div>
```

---

## 17. Empty State

Centered placeholder shown when a list has no items.

| Class                | Purpose                                            |
| -------------------- | -------------------------------------------------- |
| `.empty-state`       | Flex column, centered, large padding               |
| `.empty-state-icon`  | `--font-size-xl`, 60% opacity                      |
| `.empty-state-title` | `--font-size-md`, medium weight                    |
| `.empty-state-desc`  | `--font-size-sm`, secondary color, 320px max-width |

```svelte
<div class="empty-state">
  <span class="empty-state-icon">🤖</span>
  <span class="empty-state-title">No agents yet</span>
  <span class="empty-state-desc">Create your first agent to get started.</span>
  <button class="btn btn--primary" style="margin-top: var(--spacing-md)"> + Create Agent </button>
</div>
```

---

## 18. Banners

Feedback messages for success, warning, error, and info states.

| Class              | Appearance                             |
| ------------------ | -------------------------------------- |
| `.banner--success` | Green text on 8% green bg              |
| `.banner--warning` | Copper text on 8% copper bg            |
| `.banner--error`   | Red text on 8% red bg                  |
| `.banner--info`    | Secondary text, secondary bg, bordered |

```svelte
<div class="banner banner--success">✓ Connection successful</div>
<div class="banner banner--error">✕ Connection failed: timeout</div>
```

---

## 19. Code Snippet

Inline code display with copy button (e.g., MCP server URLs, command snippets).

| Class                | Purpose                                             |
| -------------------- | --------------------------------------------------- |
| `.code-snippet`      | Flex row: code + copy button. Tertiary bg, bordered |
| `.code-snippet code` | Monospace text, word-break, padding                 |
| `.code-snippet-copy` | 36px-wide copy button on the right, border-left     |

```svelte
<div class="code-snippet">
  <code>npx -y @azure/mcp@3.0.0 server start</code>
  <button class="code-snippet-copy" onclick={copy}>📋</button>
</div>
```

---

## 20. Segmented Control & Tab Bar

### Segmented Control

Small inline toggle for mode switching (e.g., HTTP/Stdio transport).

| Class                     | Purpose                                   |
| ------------------------- | ----------------------------------------- |
| `.segmented`              | Container with border, tiny inner padding |
| `.segmented-item`         | Individual option                         |
| `.segmented-item--active` | Secondary bg, primary text, shadow        |

### Tab Bar

Pill-shaped navigation tabs. Available for components that need tabbed navigation
(currently unused — Settings panel was migrated to sections layout).

| Class               | Purpose                               |
| ------------------- | ------------------------------------- |
| `.tab-bar`          | Flex row with bottom border, wrapping |
| `.tab-pill`         | Pill button with border               |
| `.tab-pill--active` | Secondary bg, semibold, shadow        |

---

## 21. Spinner & Progress

### Spinner

| Class          | Size    | Usage                     |
| -------------- | ------- | ------------------------- |
| `.spinner`     | 16×16px | Default loading indicator |
| `.spinner--sm` | 12×12px | Inline/small loading      |

### Progress Bar

| Class            | Purpose                                |
| ---------------- | -------------------------------------- |
| `.progress`      | 6px track, tertiary bg, rounded        |
| `.progress-fill` | Copper fill, animated width transition |

```svelte
<div class="progress">
  <div class="progress-fill" style="width: {percent}%"></div>
</div>
```

---

## 22. Checkbox List

Used for skill/MCP assignment in agent forms.

| Class               | Purpose                                   |
| ------------------- | ----------------------------------------- |
| `.check-list`       | Flex column container                     |
| `.check-item`       | Flex row: checkbox + label/desc. Hover bg |
| `.check-item-label` | `--font-size-sm`, primary color           |
| `.check-item-desc`  | `--font-size-xs`, tertiary color          |

```svelte
<div class="check-list">
  <label class="check-item">
    <input type="checkbox" checked />
    <div>
      <span class="check-item-label">Web Search</span>
      <span class="check-item-desc">Search the web via Bing/Google API</span>
    </div>
  </label>
</div>
```

---

## 23. File Item

File attachment display (projects, chat).

| Class             | Purpose                                                |
| ----------------- | ------------------------------------------------------ |
| `.file-item`      | Flex row: icon + info + action. Secondary bg, bordered |
| `.file-item-icon` | Flex-shrink emoji/icon                                 |
| `.file-item-info` | Flex column, min-width 0 (truncation)                  |
| `.file-item-name` | `--font-size-sm`, ellipsis                             |
| `.file-item-meta` | `--font-size-xs`, tertiary                             |

---

## 24. Form Actions

Bottom bar for form submit/cancel buttons.

```css
.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  padding-top: var(--spacing-md);
  margin-top: var(--spacing-md);
  border-top: 1px solid var(--color-border-secondary);
}
```

```svelte
<div class="form-actions">
  <button class="btn" onclick={cancel}>Cancel</button>
  <button class="btn btn--primary">Save Agent</button>
</div>
```

---

## 25. Kbd

Keyboard shortcut display.

```svelte
<kbd class="kbd">⌘</kbd> + <kbd class="kbd">N</kbd>
```

Mono font, bordered, subtle shadow to mimic physical key.

---

## 26. Slash Command Popup

Floating popover for `/` prefix command completions in `InputArea`.

```svelte
<div class="slash-popup">
  {#each filtered as cmd, i}
    <button class="slash-popup-item" class:selected={i === selectedIndex}>
      <span class="slash-popup-name">/{cmd.name}</span>
      {#if cmd.aliases?.length}
        <span class="slash-popup-alias">· /{cmd.aliases[0]}</span>
      {/if}
      <span class="slash-popup-desc">{cmd.description}</span>
    </button>
  {/each}
</div>
```

| Token         | Value                          | Usage                               |
| ------------- | ------------------------------ | ----------------------------------- |
| Background    | `var(--color-bg-secondary)`    | Popup container                     |
| Border        | `var(--color-border)`          | Popup border                        |
| Selected bg   | `var(--color-bg-hover)`        | Highlighted command row             |
| Name text     | `var(--color-text-primary)`    | Command name (e.g., `/help`)        |
| Alias text    | `var(--color-text-secondary)`  | Alias hint (e.g., `· /web`) at 13px |
| Description   | `var(--color-text-secondary)`  | Command description                 |
| Shadow        | `var(--shadow-lg)`             | Popup elevation                     |
| Border radius | `var(--radius-md)`             | Popup corners                       |
| Max width     | Constrained to input box width | Popup positioning                   |

**Keyboard interaction:** ↑/↓ navigate (wraps around at edges), Tab or click to accept, Escape to dismiss. Typing filters the list in real time.

---

## 27. Scroll-to-Bottom Button

Floating action button in `ChatView` that appears when the user has scrolled away from the bottom.

```svelte
{#if userScrolledAway}
  <button class="scroll-to-bottom-btn" onclick={scrollToBottom}>↓</button>
{/if}
```

| Token         | Value                                      | Usage                    |
| ------------- | ------------------------------------------ | ------------------------ |
| Background    | `var(--color-bg-secondary)`                | Button fill              |
| Border        | `var(--color-border)`                      | Button border            |
| Text          | `var(--color-text-primary)`                | Arrow icon               |
| Hover bg      | `var(--color-bg-hover)`                    | Hover state              |
| Shadow        | `var(--shadow-md)`                         | Button elevation         |
| Border radius | `50%`                                      | Circular shape           |
| Size          | `36px × 36px`                              | Button dimensions        |
| Position      | Absolute, bottom-right of chat scroll area | Above the floating input |
| Transition    | `opacity 200ms ease`                       | Fade in/out              |

---

## 28. Panel-Specific Scoped Styles

Each component keeps **only** styles that are unique to its layout and cannot be generalized.
These remain in Svelte's `<style>` block (scoped). The global unified classes handle everything else.

### What Stays Scoped

| Component         | Scoped Styles                                                                                                                                             | Reason                                                                    |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| **AgentsPanel**   | Avatar picker (emoji grid dropdown), expand button rotation, agent form fieldsets, catalog collapsible sections, registry/git result cards, progress area | Avatar picker is unique to Agents; collapsible catalog is complex layout  |
| **SkillsPanel**   | MCP group headers, skill expand/collapse, registry card hover, create-skill form, install count, git file cards, collapsible sections                     | Skill grouping by MCP server is unique; registry expand is custom         |
| **McpSettings**   | Server error display, tools toggle/list, registry entry layout, detail body/meta/links, setup commands, inline code                                       | MCP detail view has unique multi-column detail layout                     |
| **McpServerForm** | Prefill notice, server form layout, split form actions                                                                                                    | Minimal — only 6 selectors. Prefill notice is unique to registry pre-fill |
| **SettingsPanel** | Account card (avatar + details), shortcut rows, cleanup controls, export buttons, settings card variants, tab pill SVG styling                            | Account display is unique; shortcut key display is custom                 |
| **ProjectView**   | Project card grid, chevron, inline edit, conversation list, assign modal, danger zone, file list actions                                                  | Project card layout and assign modal are unique                           |
| **InputArea**     | Slash command popup, @-mention popup, command item highlight, alias text styling, help modal layout                                                       | Slash popup is unique to InputArea; popup positioning relative to input   |
| **ChatView**      | Scroll-to-bottom button positioning, fade-in animation, floating placement above input area                                                               | Button must be positioned absolutely within the chat scroll container     |

### Rule of Thumb

Before adding a scoped style, check if the global system already provides it:

1. **Is it a button?** → Use `.btn` + modifiers
2. **Is it a card?** → Use `.card` + modifiers
3. **Is it a form field?** → Use `.form-field` + `.form-input`
4. **Is it a badge/label?** → Use `.badge` + variant
5. **Is it a loading state?** → Use `.spinner` or `.empty-state`
6. **Is it a feedback message?** → Use `.banner` + variant

If none of these fit, create a scoped style — but name it descriptively and keep it minimal.

---

## 29. Do's and Don'ts

### ✅ Do

- **Use design tokens** for all values: `var(--color-text-primary)`, not `#1c1917`
- **Use global classes** for standard components (buttons, cards, forms, badges)
- **Use `gap`** on flex/grid parents for spacing, not `margin` on children
- **Use semantic HTML** (`<button>`, `<label>`, `<fieldset>`, `<nav>`)
- **Use `aria-label`** on icon-only buttons
- **Test both themes** — verify every component in light and dark mode
- **Use `color-mix()`** for semi-transparent semantic backgrounds
- **Combine modifiers** — `.btn.btn--ghost.btn--sm` is the correct pattern
- **Use `.card--featured`** for built-in/default items (one per panel, typically)

### ❌ Don't

- **Don't hardcode colors** — no `#b45309`, always `var(--color-accent-copper)`
- **Don't use inline styles** — exception only for truly dynamic values (progress width, animation delay)
- **Don't duplicate global classes** in scoped `<style>` blocks
- **Don't use `.form-field + .form-field`** for spacing — use `gap` on parent
- **Don't create new button variants** — compose with existing modifiers
- **Don't use `--font-size-base`** in components — it's for the root `<body>` only
- **Don't mix Instrument Serif with non-italic** — display font is always italic
- **Don't add hover effects to disabled elements** — all `:hover` rules use `:not(:disabled)`
- **Don't use `any` type** in TypeScript — use proper types for all component props/state
- **Don't create component-specific color tokens** — use the existing palette

### Common Mistakes

| Mistake                       | Correct Approach                                |
| ----------------------------- | ----------------------------------------------- |
| `<div onclick={...}>`         | `<button class="btn btn--ghost" onclick={...}>` |
| `style="color: red"`          | `class="banner banner--error"`                  |
| `.my-toggle { width: 40px }`  | Use `.toggle` (unified 36×20px)                 |
| `.my-btn { ... }` (new class) | Compose: `.btn.btn--primary.btn--sm`            |
| `margin-bottom: 8px` on items | `gap: var(--spacing-sm)` on parent flex         |
| `font-size: 11px`             | `font-size: var(--font-size-xs)`                |

---

## 30. Agents Panel Reference

### Views

The Agents panel has 3 views sharing one `.panel` container:

1. **List View** — Panel header ("Agents") + agent cards
2. **Create View** — Panel header (chevron back + "Create Agent") + form
3. **Edit View** — Panel header (chevron back + "Edit Agent") + pre-filled form

### Agent Card Structure

```
┌─ .card .card--clickable (.card--featured for default) ──────────┐
│  ┌─ .card-header ─────────────────────────────────────────────┐  │
│  │ .card-icon   .card-title   .card-actions [Edit] [Delete]  │  │
│  └────────────────────────────────────────────────────────────┘  │
│  .card-desc — "Deep research with web search and citations."    │
│  ┌─ .card-meta ───────────────────────────────────────────────┐  │
│  │ .badge--copper "2 skills" │ .badge--neutral "aitmpl.com"  │  │
│  └────────────────────────────────────────────────────────────┘  │
│  ┌─ .card-detail (expanded) ──────────────────────────────────┐  │
│  │ .detail-section: System Prompt                             │  │
│  │ .detail-section: Skills                                    │  │
│  │ .detail-section: MCP Connections                           │  │
│  │ .detail-section: Source                                    │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

### Agent Form Structure

```
┌─ .panel-body-narrow ─────────────────────────────────────────────┐
│  .form-field: Avatar (.avatar-picker) + Name (inline flex)       │
│  .form-field: System Prompt (textarea.form-input--mono)          │
│  .form-fieldset: Assigned Skills (.check-list)                   │
│  .form-fieldset: MCP Connections (.check-list)                   │
│  .form-actions: [Cancel] [Save Agent .btn--primary]              │
└──────────────────────────────────────────────────────────────────┘
```

### Scoped Classes (agents-only)

| Class                  | Purpose                                            |
| ---------------------- | -------------------------------------------------- |
| `.expand-btn`          | Arrow chevron that rotates 180° when expanded      |
| `.avatar-picker`       | Emoji grid dropdown (trigger + dropdown + grid)    |
| `.avatar-trigger`      | Button that shows current emoji + caret            |
| `.avatar-dropdown`     | Absolutely-positioned emoji grid                   |
| `.avatar-option`       | Individual emoji in the grid                       |
| `.agent-form`          | Form container with `gap: --spacing-lg`            |
| `.form-row`            | Inline flex row (avatar + name side by side)       |
| `.form-fieldset`       | Styled `<fieldset>` wrapper for skill/MCP sections |
| `.catalog-section`     | Collapsible registry/git import section            |
| `.collapsible-heading` | Clickable section header with arrow toggle         |

---

## 31. Skills Panel Reference

### Views

1. **List View** — Grouped by source: Built-in → MCP (per server) → Registry/Git → Extensions
2. **Create Skill Form** — Inline form at the bottom

### Skill Card Structure

```
┌─ .card .card--clickable ────────────────────────────────────────┐
│  ┌─ .card-header ─────────────────────────────────────────────┐  │
│  │  .toggle  .card-title  .card-actions [expand-btn]          │  │
│  └────────────────────────────────────────────────────────────┘  │
│  .card-desc — "Search the web via Bing/Google API"              │
│  .card-meta — .badge--neutral "built-in" │ .badge--copper link  │
│  ┌─ .card-detail (expanded) ──────────────────────────────────┐  │
│  │  .detail-section: Instructions (markdown body)             │  │
│  │  .detail-section: Source URL                               │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

### Scoped Classes (skills-only)

| Class                 | Purpose                                       |
| --------------------- | --------------------------------------------- |
| `.group-heading`      | MCP server group name header                  |
| `.mcp-group`          | Container for skills from one MCP server      |
| `.skill-info`         | Inline flex for toggle + title                |
| `.skill-expand-btn`   | Expand/collapse chevron                       |
| `.skill-instructions` | Rendered instruction content                  |
| `.registry-card`      | Registry result with hover + expand           |
| `.install-count`      | Download count display                        |
| `.create-skill-form`  | Bottom form section for manual skill creation |

---

## 32. MCP Panel Reference

### Views

1. **Server List** — Connected servers + MCP Registry browser
2. **Add/Edit Server** — `McpServerForm.svelte` (separate component)

### MCP Server Card Structure

```
┌─ .card .card--clickable ────────────────────────────────────────┐
│  ┌─ .card-header ─────────────────────────────────────────────┐  │
│  │  .status (🟢/🔴)  .card-title  .card-actions [Test][Edit] │  │
│  └────────────────────────────────────────────────────────────┘  │
│  .card-desc — "Transport: HTTP │ URL: https://..."              │
│  .card-meta — .badge--mono "HTTP" │ .badge--copper "12 tools"  │
│  ┌─ .card-detail (expanded) ──────────────────────────────────┐  │
│  │  Tool list, connection details, error messages              │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

### McpServerForm Pattern

This is the **canonical reference implementation** for how to use global classes in a form view:

| Old Scoped Class       | New Global Class          |
| ---------------------- | ------------------------- |
| `.form-page`           | `.panel`                  |
| `.form-header`         | `.panel-header`           |
| `.back-btn`            | `.panel-back`             |
| `.form-title`          | `.panel-title`            |
| `.form-content`        | `.panel-body-narrow`      |
| `.field-label`         | `.form-label`             |
| `.field-hint`          | `.form-hint`              |
| `.radio-group`         | `.form-radio-group`       |
| `.action-btn`          | `.btn`                    |
| `.action-btn.primary`  | `.btn.btn--primary`       |
| `.test-result.success` | `.banner.banner--success` |
| `.test-result.failure` | `.banner.banner--error`   |

### Scoped Classes (MCP-only)

| Class                        | Purpose                                      |
| ---------------------------- | -------------------------------------------- |
| `.tools-toggle`              | Toggle to show/hide discovered tools list    |
| `.tools-list` / `.tool-item` | Discovered MCP tools display                 |
| `.registry-entry`            | MCP Registry search result row               |
| `.detail-body`               | Expanded registry detail view                |
| `.setup-commands`            | Command display for server setup             |
| `.prefill-notice`            | Banner when form is pre-filled from registry |

---

## 33. Settings Panel Reference

### Layout

Settings uses a **single scrollable sections layout** (no tabs). All settings are grouped
under `<h2 class="section-heading">` headings with `.settings-card` containers:

**Section order:** Account → Appearance → Input → Defaults → Auto-Update → Keyboard Shortcuts → Storage → Cleanup → Export

### Section Pattern

Each settings section uses `.section-heading` + `.settings-card` + repeated `.setting-row` entries:

```svelte
<h2 class="section-heading">Appearance</h2>

<div class="settings-card">
  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Theme</span>
      <span class="setting-desc">Choose your preferred appearance</span>
    </div>
    <select class="form-select">
      <option>System</option>
      <option>Light</option>
      <option>Dark</option>
    </select>
  </div>

  <div class="setting-row">
    <div class="setting-info">
      <span class="setting-label">Auto-check for updates</span>
    </div>
    <label class="toggle">
      <input type="checkbox" />
      <span class="toggle-track"></span>
    </label>
  </div>
</div>
```

### Scoped Classes (settings-only)

| Class               | Purpose                                         |
| ------------------- | ----------------------------------------------- |
| `.account-card`     | Account info display (avatar + username + plan) |
| `.account-header`   | Flex row for avatar + details                   |
| `.account-avatar`   | User avatar image/placeholder                   |
| `.shortcuts-list`   | Keyboard shortcuts grid                         |
| `.shortcut-row`     | Action label + `.kbd` key display               |
| `.cleanup-controls` | DB cleanup date selector + button               |
| `.export-buttons`   | Export format button group                      |
| `.settings-card`    | Wrapper card for grouped settings               |

---

## 34. Projects Panel Reference

### Views

1. **List View** — Project cards with name, stats, chevron
2. **Detail View** — Project name, instructions, files, conversations

### Project Card (List)

```
┌─ .card .card--clickable (.project-card) ────────────────────────┐
│  .card-header: .card-icon 📁 + .card-title + chevron ›          │
│  .card-meta: .badge--copper "3 files" │ .badge--neutral "5 chats"│
└──────────────────────────────────────────────────────────────────┘
```

### Project Detail

Uses `.panel-body-narrow` with `.detail-section` blocks:

- Instructions (editable textarea)
- Files (`.file-item` list with add/remove)
- Conversations (linked conversation list)
- Danger zone (delete project)

### Scoped Classes (projects-only)

| Class                            | Purpose                                      |
| -------------------------------- | -------------------------------------------- |
| `.project-card`                  | Card variant with full-width click + chevron |
| `.project-card-chevron`          | Right-aligned › arrow                        |
| `.inline-edit`                   | Inline text editing wrapper                  |
| `.conversation-list`             | Linked conversations display                 |
| `.conv-item` / `.conv-item-link` | Conversation row with hover                  |
| `.assign-list` / `.assign-item`  | Modal for assigning conversations to project |
| `.danger-zone`                   | Red-bordered delete section                  |

---

## Appendix: Full Class Map

### Global Classes (in `src/app.css`)

```
Panel:          .panel, .panel-header, .panel-back, .panel-title, .panel-body, .panel-body-narrow
Sections:       .section-heading
Cards:          .card, .card--featured, .card--clickable, .card--flat,
                .card-header, .card-icon, .card-title, .card-actions, .card-desc, .card-meta
Detail:         .card-detail, .detail-section, .detail-label, .detail-value, .detail-value--muted
Buttons:        .btn, .btn--primary, .btn--accent, .btn--ghost, .btn--link, .btn--danger,
                .btn--pill, .btn--sm, .btn--icon
Badges:         .badge, .badge--neutral, .badge--copper, .badge--success, .badge--error, .badge--mono
Status:         .status, .status-dot, .status--connected, .status--disconnected, .status--warning
Toggle:         .toggle, .toggle-track
Forms:          .form-field, .form-label, .form-hint, .form-input, .form-input--mono,
                .form-select, .form-radio-group, .form-error
Settings:       .setting-row, .setting-info, .setting-label, .setting-desc
Search:         .search-field, .search-field-icon, .search-spinner
Empty:          .empty-state, .empty-state-icon, .empty-state-title, .empty-state-desc
Banners:        .banner, .banner--success, .banner--warning, .banner--error, .banner--info
Code:           .code-snippet, .code-snippet code, .code-snippet-copy
Segmented:      .segmented, .segmented-item, .segmented-item--active
Tabs:           .tab-bar, .tab-pill, .tab-pill--active
Spinner:        .spinner, .spinner--sm
Kbd:            .kbd
Progress:       .progress, .progress-fill
Checkboxes:     .check-list, .check-item, .check-item-label, .check-item-desc
Actions:        .form-actions
Files:          .file-item, .file-item-icon, .file-item-info, .file-item-name, .file-item-meta
```
