# Markdown to Typst Syntax Mapping

This document provides a comprehensive reference for how Markdown syntax is mapped to equivalent Typst syntax. This is useful for understanding how Markdown content is converted when using tools like mdbook-typstpdf.

## Basic Elements

### Headings

| Markdown | Typst |
|----------|-------|
| `# Heading 1` | `= Heading 1` |
| `## Heading 2` | `== Heading 2` |
| `### Heading 3` | `=== Heading 3` |
| `#### Heading 4` | `==== Heading 4` |
| `##### Heading 5` | `===== Heading 5` |
| `###### Heading 6` | `====== Heading 6` |

Alternative setext-style headings in Markdown (not recommended):

```markdown
Heading 1
=========

Heading 2
---------
```

### Paragraphs

| Markdown | Typst |
|----------|-------|
| `Text with a line break in between creates a new paragraph.` | `Text with a line break in between creates a new paragraph.` |
| Line breaks require an empty line between paragraphs | Same in Typst - empty line creates paragraph breaks |

### Line Breaks

| Markdown | Typst |
|----------|-------|
| `Line with two spaces at end  ` <br> `Next line` | `Line with a line break \` <br> `Next line` |
| `Line with backslash at end\` <br> `Next line` | `Line with a line break \` <br> `Next line` |

## Text Formatting

### Emphasis

| Markdown | Typst |
|----------|-------|
| `*italic*` or `_italic_` | `_italic_` |
| `**bold**` or `__bold__` | `*bold*` |
| `***bold and italic***` | `*_bold and italic_*` |
| `~~strikethrough~~` (GFM extension) | `#strike[strikethrough]` |

### Superscript and Subscript

| Markdown | Typst |
|----------|-------|
| `H<sub>2</sub>O` (HTML) | `H#sub[2]O` |
| `X<sup>2</sup>` (HTML) | `X#super[2]` |

## Lists

### Unordered Lists

| Markdown | Typst |
|----------|-------|
| `- Item 1`<br>`- Item 2`<br>`  - Nested Item` | `- Item 1`<br>`- Item 2`<br>`  - Nested Item` |
| `* Item 1`<br>`* Item 2` | `- Item 1`<br>`- Item 2` |
| `+ Item 1`<br>`+ Item 2` | `- Item 1`<br>`- Item 2` |

### Ordered Lists

| Markdown | Typst |
|----------|-------|
| `1. Item 1`<br>`2. Item 2`<br>`   1. Nested Item` | `+ Item 1`<br>`+ Item 2`<br>`  + Nested Item` |
| `1. Item 1`<br>`1. Item 2` (auto-numbering) | `+ Item 1`<br>`+ Item 2` |

### Task Lists (GFM Extension)

| Markdown | Typst |
|----------|-------|
| `- [ ] Todo item` | Custom implementation using `#task-list()` function |
| `- [x] Completed item` | Custom implementation using `#task-list()` function |

## Links and Images

### Links

| Markdown | Typst |
|----------|-------|
| `[Link text](https://example.com)` | `#link("https://example.com")[Link text]` |
| `[Link with title](https://example.com "Title")` | `#link("https://example.com")[Link text]` |
| `<https://example.com>` | `#link("https://example.com")` |
| `[Reference link][ref]`<br><br>`[ref]: https://example.com` | `#link("https://example.com")[Reference link]` |

### Images

| Markdown | Typst |
|----------|-------|
| `![Alt text](image.jpg)` | `#figure(image("image.jpg"), caption: [Alt text])` |
| `![Alt text](image.jpg "Title")` | `#figure(image("image.jpg"), caption: [Alt text])` |
| `[![Alt text](image.jpg)](https://example.com)` | `#link("https://example.com")[#image("image.jpg")]` |

## Code

### Inline Code

| Markdown | Typst |
|----------|-------|
| `` `inline code` `` | `` `inline code` `` |

### Code Blocks

| Markdown | Typst |
|----------|-------|
| ````<br>```<br>code block<br>```<br>```` | ```rust<br>#raw(block: true, lang: "none", ```<br>code block<br>```)<br>``` |
| ````<br>```rust<br>fn main() {}<br>```<br>```` | ```rust<br>#raw(block: true, lang: "rust", ```<br>fn main() {}<br>```)<br>``` |

## Blockquotes

| Markdown | Typst |
|----------|-------|
| `> This is a quote`<br>`> Multiple lines` | `#quote[This is a quote. Multiple lines]` |
| `> Nested`<br>`>> Quotes` | `#quote[Nested #quote[Quotes]]` |

## Horizontal Rules

| Markdown | Typst |
|----------|-------|
| `---` or `***` or `___` | `#line(length: 100%)` |

## Tables

| Markdown | Typst |
|----------|-------|
| `\| Header 1 \| Header 2 \|`<br>`\| -------- \| -------- \|`<br>`\| Cell 1   \| Cell 2   \|` | ```#table(`<br>`  columns: (auto, auto),`<br>`  [Header 1], [Header 2],`<br>`  [Cell 1], [Cell 2],`<br>`)<br>``` |
| `\| Left \| Center \| Right \|`<br>`\|:---- \|:----:\| ----:\|`<br>`\| Text \| Text  \| Text \|` | ```#table(`<br>`  columns: (auto, auto, auto),`<br>`  align: (left, center, right),`<br>`  [Left], [Center], [Right],`<br>`  [Text], [Text], [Text],`<br>`)<br>``` |

## Special Characters and Escaping

| Markdown | Typst |
|----------|-------|
| `\*Not italic\*` | `\*Not italic\*` |
| `\`Not code\`` | `\`Not code\`` |
| `\\Backslash` | `\\Backslash` |

Special Typst characters that need escaping when converted from Markdown:
- `#` (hash/pound) → `\#`
- `[` (left square bracket) → `\[`
- `]` (right square bracket) → `\]`
- `$` (dollar sign) → `\$`
- `` ` `` (backtick) → ``\` ``
- `*` (asterisk) → `\*`
- `_` (underscore) → `\_`
- `\` (backslash) → `\\`

## Math

| Markdown | Typst |
|----------|-------|
| `$inline math$` | `$inline math$` |
| `$$display math$$` | `$ display math $` |

## Footnotes

| Markdown | Typst |
|----------|-------|
| `Text with a footnote[^1]`<br><br>`[^1]: Footnote content` | `Text with a footnote#footnote[Footnote content]` |

## Comments

| Markdown | Typst |
|----------|-------|
| `<!-- This is a comment -->` | `#comment[This is a comment]` |

## Special Typst Features

These are Typst-specific features without direct Markdown equivalents:

### Page Breaks
```typst
#pagebreak()
```

### Raw Content
```typst
#raw(block: true, lang: "html", "<div>HTML content</div>")
```

### Custom Styling
```typst
#set heading(numbering: "1.1")
#set text(font: "New Computer Modern")
```

## Image Sizing

In mdbook-typstpdf, images are dynamically sized based on their dimensions:
- Images wider than 60% of the page width are scaled to 60% width
- Images with height > 0.6 × page height are scaled proportionally
- Small images are kept at their original size

## Conversion Notes

1. Markdown reference-style links are converted to inline Typst links
2. HTML tags in Markdown are generally preserved as raw HTML in Typst
3. Special characters are escaped to prevent conflicts with Typst syntax
4. Some GFM (GitHub Flavored Markdown) extensions may require custom implementation in Typst
