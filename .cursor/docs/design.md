# Design System

Use a small, portable design system that works as plain HTML/CSS/JS on the Internet Archive. Do not add external font, icon, CSS framework, or CDN dependencies.

## Typography

- Use the web-safe font stack `Arial, sans-serif` for all UI text.
- Keep copy readable with generous line height, short labels, and clear form/help text.

## Color Tokens

Use the Tailwind color values from Material UI as project tokens.
https://materialui.co/tailwindcolors

Primary is orange, neutral is zinc.

```css
:root { 
    --color-primary-50: #fff7ed;
    --color-primary-100: #ffedd5;
    --color-primary-200: #fed7aa;
    --color-primary-300: #fdba74;
    --color-primary-400: #fb923c;
    --color-primary-500: #f97316;
    --color-primary-600: #ea580c;
    --color-primary-700: #c2410c;
    --color-primary-800: #9a3412;
    --color-primary-900: #7c2d12;
    --color-primary-950: #431407;
    --color-primary-gradient: linear-gradient(to bottom right, var(--color-primary-100), var(--color-primary-600), var(--color-primary-800));

    --color-neutral-0 : #ffffff;
    --color-neutral-50: #fafafa;
    --color-neutral-100: #f4f4f5;
    --color-neutral-200: #e4e4e7;
    --color-neutral-300: #d4d4d8;
    --color-neutral-400: #a1a1aa;
    --color-neutral-500: #71717a;
    --color-neutral-600: #52525b;
    --color-neutral-700: #3f3f46;
    --color-neutral-800: #27272a;
    --color-neutral-900: #18181b;
    --color-neutral-950: #09090b;
    --color-neutral-1000: #000000;
    --color-neutral-gradient: linear-gradient(to bottom, var(--color-neutral-700), var(--color-neutral-900)); /* used for borders */

    /* status colors - equals --color-primary-600 tint */
    --color-status-red: #dc2626;
    --color-status-green: #16a34a;
    --color-status-orange: #ea580c;
    --color-status-blue: #2563eb; 

    /* size - based on tailwind css standard */
    --size-0-5: 0.125rem; /* 2px */
    --size-1: 0.25rem; /* 4px */
    --size-1-5: 0.375rem; /* 6px */
    --size-2: 0.5rem; /* 8px */
    --size-2-5: 0.625rem; /* 10px */  
    --size-3: 0.75rem; /* 12px */
    --size-3-5: 0.875rem; /* 14px */  
    --size-4: 1rem; /* 16px */
    --size-5: 1.25rem; /* 20px */
    --size-6: 1.5rem; /* 24px */
    --size-7: 1.75rem; /* 28px */
    --size-8: 2rem; /* 32px */
    --size-9: 2.25rem; /* 36px */
    --size-10: 2.5rem; /* 40px */  

    --font-sans: Arial, sans-serif;
    --shadow-card: 0 1rem 3rem rgba(9, 9, 11, 0.12);
}
```

## CSS Classes

```css

/* ===== Keyboard focus - all focusable elements ===== */
:focus-visible {
  outline: var(--size-0-5) solid var(--color-primary-400);
  outline-offset: var(--size-0-5);
  border-radius: var(--size-3);
}


/* ===================================================================
   BODY STYLES
   =================================================================== */
* {
    box-sizing: border-box;
}

html {
    color: var(--color-neutral-300);
    background: var(--color-neutral-800);
    font-family: var(--font-sans);
    line-height: 1.5;
}

body {
    display: flex;
    justify-content: center;
    min-width: 20rem;
    min-height: 100vh;
    padding: var(--size-6);
}

/* ===================================================================
   CONTAINERS
   =================================================================== */

/* ===== Base containers ===== */
.container-layer-1 {
    display: flex;
    flex-direction: column;
    width: min(100%, 64rem);
    height: fit-content;
    margin: var(--size-4);
    padding: var(--size-8) var(--size-8);
    gap: var(--size-6);
    border: 0.075rem solid transparent;
    border-radius: var(--size-6);
    background:
        linear-gradient(var(--color-neutral-950), var(--color-neutral-1000)) padding-box,
        var(--color-neutral-gradient) border-box;
    box-shadow: var(--shadow-card);
    overflow: hidden;
}

.container-layer-2 {
    display: flex;
    flex-direction: column;
    width: 100%;
    padding: var(--size-4) var(--size-6);
    border: 0.075rem solid transparent;
    border-radius: var(--size-4);
    background:
        linear-gradient(to bottom, var(--color-neutral-1000) 60%, var(--color-neutral-900) 100%) padding-box,
        var(--color-neutral-gradient) border-box;
    overflow: hidden;
}

/* ===== Pill container ===== */
.container-pill {
    width: fit-content;
    padding: var(--size-1) var(--size-2);
    border-radius: 999px;
    color: var(--color-primary-600);
    background-color: var(--color-neutral-0);
    font-size: var(--size-2-5);
    font-weight: 600;
    letter-spacing: 0.1rem;
}

/* ===== Row/column containers ===== */
.container-row {
    display: flex;
    justify-content: space-between;
}

.container-row--center {
    align-items: center;
}

.container-column {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-height: 0;
}

/* ===================================================================
   TEXT STYLES
   =================================================================== */

/* ===== Heading ===== */
h1,
h2,
p {
    margin-block-start: 0;
}

h1 {
    margin-block-end: var(--size-4);
    font-size: clamp(2.5rem, 10vw, 5rem);
    background: var(--color-primary-gradient);
    background-clip: text;
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    line-height: 0.95;
}

h2 {
    margin-block-end: var(--size-4);
    font-size: clamp(1.35rem, 4vw, 1.75rem);
    color: var(--color-primary-400);
}

/* ===== Caption ===== */
.caption {
    margin: 0 0 var(--size-1);
    font-size: var(--size-2-5);
    font-weight: 700;
    color: var(--color-neutral-600);
    text-transform: uppercase;
    letter-spacing: 0.1em;
}

.sub-caption {
    font-weight: 700;
    color: var(--color-neutral-500);
}

/* ===== Label ===== */
.label {
    font-size: var(--size-4);
    font-weight: 400;
    color: var(--color-neutral-500);
}

/* ===== Value ===== */
.value {
    font-size: var(--size-4);
    font-weight: 400;
    color: var(--color-primary-600);
    text-transform: uppercase;
}

.value-lg {
    font-size: var(--size-6);
    font-weight: 700;
    color: var(--color-primary-600);
}

/* ===== Helper text ===== */
.field-help,
.field-error,
.status-message,
.empty-state,
.result-provider,
.media-preview figcaption {
    font-size: var(--size-3-5);
    color: var(--color-neutral-400);
}

```

## UI Rules

- Use primary orange for focused controls, calls to action, and selected states.
- Use neutral zinc for backgrounds, surfaces, borders, body text, and subdued metadata.
- Preserve accessible contrast: avoid orange text on white below `700` for body-sized text.
- Keep focus states visible with a primary outline or ring.
- Prefer CSS custom properties over hard-coded color values in components.
