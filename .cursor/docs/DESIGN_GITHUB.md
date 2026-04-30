# GitHub README theming (Primer)

Colors below match GitHub’s **default `light` and `dark`** themes from **[@primer/primitives](https://www.npmjs.com/package/@primer/primitives)** (`dist/css/functional/themes/light.css` and `dark.css`). Use them when designing README hero images that should sit on the rendered README canvas.

## Main content area (body text + canvas)

| Mode  | Role                    | CSS variable (semantic) | Hex       |
| ----- | ----------------------- | ------------------------ | --------- |
| Light | Default text (headings, body) | `--fgColor-default`      | `#1f2328` |
| Light | Page / content background     | `--bgColor-default`      | `#ffffff` |
| Dark  | Default text                  | `--fgColor-default`      | `#f0f6fc` |
| Dark  | Page / content background     | `--bgColor-default`      | `#0d1117` |

## Nearby surfaces (light mode)

The **file header** strip (e.g. “README.md” above the content) often uses a muted surface such as **`#f6f8fa`** (`--bgColor-muted` / related control tokens), not `#ffffff`. Do not assume the whole viewport is one flat white.

## Eyedropper vs tokens

- Published Primer **dark** foreground is **`#f0f6fc`**. Some screenshots or older notes show **`#e6edf3`**; it is close but not the current `fgColor-default` in default `dark`.

## Further reading

- [Primer primitives — getting started](https://primer.style/foundations/primitives/getting-started)
- [primer/primitives README](https://github.com/primer/primitives/blob/main/README.md) (theme CSS import paths)
