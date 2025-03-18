# pulldown-cmark 0.13.0 Markdown Rules

This document outlines the parsing rules and event generation behavior of the pulldown-cmark 0.13.0 crate, highlighting notable differences from version 0.9.3 when processing Markdown content.

## Overview

pulldown-cmark is a CommonMark-compliant Markdown parser for Rust that follows the [CommonMark Spec](https://spec.commonmark.org/). Version 0.13.0 includes updates to match spec 0.29 with various extensions including GitHub Flavored Markdown (GFM) features.

## Key Changes in 0.13.0 from 0.9.3

Since 0.9.3, these significant changes were implemented up to 0.13.0:

1. **Link Reference Definition nodes** are now part of the document AST (not rendered by default)
2. **New API**: `InlineParserContext.getLinkReferenceDefinition` was added to allow custom inline parsers to look up definitions for reference links
3. **Performance improvements**:
   - Parsing 7-10% faster
   - HTML rendering 105% faster (roughly twice as fast)
4. **Table parsing behavior changes** to match GitHub's implementation:
   - Escaping now only considers pipe characters when parsing tables: `\|` results in a literal `|`
   - Table bodies can now contain lazy continuation lines (without `|`)
   - For tables without a body, `<tbody>` is no longer rendered in HTML
5. **HTML entity handling improvements**

## Markdown Element Parsing

### Block Elements

#### Headings

ATX-style headings (with leading `#` characters) are parsed as in CommonMark spec 0.29:

```markdown
# Heading 1
## Heading 2
### Heading 3
```

Setext-style headings (underlined) are properly recognized:

```markdown
Heading 1
=========

Heading 2
---------
```

*Difference from 0.9.3*: The 0.13.0 parser properly handles setext headings even when they occur after reference link definitions, which was sometimes problematic in 0.9.3.

#### Paragraphs

```markdown
This is a paragraph.

This is another paragraph.
```

Paragraph continuation rules follow spec 0.29, with proper handling of list item interruption.

#### Lists

Unordered lists use `-`, `*`, or `+` markers:

```markdown
- Item 1
- Item 2
  - Nested item
```

Ordered lists use numbers followed by `.` or `)`:

```markdown
1. Item 1
2. Item 2
   1. Nested item
```

*Difference from 0.9.3*: Version 0.13.0 has improved handling of nested list items and ensures tight/loose list distinction matches the spec more precisely.

#### Blockquotes

```markdown
> This is a blockquote
> with multiple lines
>
> And multiple paragraphs
```

#### Code Blocks

Fenced code blocks with backticks or tildes, with optional language specifier:

````markdown
```rust
fn main() {
    println!("Hello, world!");
}
```

~~~
No language specified
~~~
````

Indented code blocks (four spaces):

```markdown
    This is an indented code block
    Still in the code block
```

*Difference from 0.9.3*: In 0.13.0, code block info strings are processed more carefully, with proper handling of backticks and tildes in tilde-fenced blocks (as per spec 0.29).

#### Thematic Breaks (Horizontal Rules)

```markdown
---

***

___
```

### Inline Elements

#### Emphasis and Strong Emphasis

```markdown
*italic* or _italic_
**bold** or __bold__
***bold italic*** or ___bold italic___
```

*Difference from 0.9.3*: Version 0.13.0 includes improvements to match the updated spec rules for emphasis parsing, especially for complex cases involving multiple adjacent delimiters.

#### Links and Images

Inline links:
```markdown
[Link text](https://example.com "Title")
```

Reference links:
```markdown
[Link text][reference]

[reference]: https://example.com "Title"
```

Autolinks:
```markdown
<https://example.com>
```

Images:
```markdown
![Alt text](image.jpg "Title")
```

*Difference from 0.9.3*: The major difference in 0.13.0 is that link reference definition nodes are now part of the document AST and accessible through the new API, allowing more sophisticated link processing.

#### Inline Code

```markdown
`inline code` with backticks
```

#### HTML

HTML blocks and inline HTML are parsed according to spec 0.29 rules:

```markdown
<div>
  This is an HTML block
</div>

This has <span>inline HTML</span>.
```

*Difference from 0.9.3*: Version 0.13.0 fixes how HTML entities are preserved when rendering attributes.

### GFM Extensions

#### Tables

```markdown
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
```

*Key differences from 0.9.3*:
- Escaping now only considers pipe characters when parsing tables: `\|` results in a literal `|` instead of a column
- Table bodies can contain lazy continuation lines (without `|`)
- For tables without a body, `<tbody>` is no longer rendered in HTML

#### Strikethrough

```markdown
~~Strikethrough text~~
```

#### Task Lists

```markdown
- [ ] Unchecked task
- [x] Checked task
```

## Event Generation

pulldown-cmark generates a sequence of events when parsing Markdown text. The main event types remain the same in 0.13.0:

- `Start(Tag)` - Beginning of a block or inline element
- `End(Tag)` - End of a block or inline element
- `Text(String)` - Plain text content
- `Code(String, Option<String>)` - Inline code with optional language info
- `Html(String)` - HTML block or inline content
- `FootnoteReference(String)` - Reference to a footnote
- `SoftBreak` - Soft line break
- `HardBreak` - Hard line break
- `Rule` - Horizontal rule/thematic break
- `TaskListMarker(bool)` - Task list marker (checked or unchecked)

*New in 0.13.0*: The parser now exposes link reference definition nodes that can be accessed through the new API.

## Rust Event Examples

This section provides concrete examples of the Rust events generated by pulldown-cmark 0.13.0 for various Markdown constructs. These examples show the actual event sequence that would be yielded when iterating over the parser.

### Basic Block Elements

#### Heading Events

For ATX headings:

```markdown
# Heading 1
```

Generated events in 0.13.0:
```rust
Event::Start(Tag::Heading(HeadingLevel::H1, None, Vec::new()))
Event::Text("Heading 1".into())
Event::End(Tag::Heading(HeadingLevel::H1, None, Vec::new()))
```

*Difference from 0.9.3*: In 0.13.0, heading tags include an identifier and attributes vector, represented by `None, Vec::new()` above. This feature supports heading attributes and identifiers, not available in 0.9.3.

#### Paragraph Events

```markdown
This is a paragraph.
```

Generated events:
```rust
Event::Start(Tag::Paragraph)
Event::Text("This is a paragraph.".into())
Event::End(Tag::Paragraph)
```

### List Events

Unordered list:
```markdown
- Item 1
- Item 2
  - Nested item
```

Generated events in 0.13.0:
```rust
Event::Start(Tag::List(None)) // No starting number indicates unordered list
Event::Start(Tag::Item)
Event::Start(Tag::Paragraph)
Event::Text("Item 1".into())
Event::End(Tag::Paragraph)
Event::End(Tag::Item)
Event::Start(Tag::Item)
Event::Start(Tag::Paragraph)
Event::Text("Item 2".into())
Event::End(Tag::Paragraph)
Event::Start(Tag::List(None)) // Nested list
Event::Start(Tag::Item)
Event::Start(Tag::Paragraph)
Event::Text("Nested item".into())
Event::End(Tag::Paragraph)
Event::End(Tag::Item)
Event::End(Tag::List(None))
Event::End(Tag::Item)
Event::End(Tag::List(None))
```

Ordered list with task items:
```markdown
1. [ ] Task 1
2. [x] Task 2
```

Generated events in 0.13.0:
```rust
Event::Start(Tag::List(Some(1))) // Starting number 1
Event::Start(Tag::Item)
Event::TaskListMarker(false) // Unchecked
Event::Start(Tag::Paragraph)
Event::Text("Task 1".into())
Event::End(Tag::Paragraph)
Event::End(Tag::Item)
Event::Start(Tag::Item)
Event::TaskListMarker(true) // Checked
Event::Start(Tag::Paragraph)
Event::Text("Task 2".into())
Event::End(Tag::Paragraph)
Event::End(Tag::Item)
Event::End(Tag::List(Some(1)))
```

### Inline Formatting Events

Emphasis and strong:
```markdown
*italic* **bold** ***both***
```

Generated events:
```rust
Event::Start(Tag::Paragraph)
Event::Start(Tag::Emphasis)
Event::Text("italic".into())
Event::End(Tag::Emphasis)
Event::Text(" ".into())
Event::Start(Tag::Strong)
Event::Text("bold".into())
Event::End(Tag::Strong)
Event::Text(" ".into())
Event::Start(Tag::Strong)
Event::Start(Tag::Emphasis)
Event::Text("both".into())
Event::End(Tag::Emphasis)
Event::End(Tag::Strong)
Event::End(Tag::Paragraph)
```

Strikethrough:
```markdown
~~deleted~~
```

Generated events:
```rust
Event::Start(Tag::Paragraph)
Event::Start(Tag::Strikethrough)
Event::Text("deleted".into())
Event::End(Tag::Strikethrough)
Event::End(Tag::Paragraph)
```

### Link Events

Inline link:
```markdown
[Link text](https://example.com "Title")
```

Generated events in 0.13.0:
```rust
Event::Start(Tag::Paragraph)
Event::Start(Tag::Link(LinkType::Inline, "https://example.com".into(), "Title".into()))
Event::Text("Link text".into())
Event::End(Tag::Link(LinkType::Inline, "https://example.com".into(), "Title".into()))
Event::End(Tag::Paragraph)
```

Reference link:
```markdown
[Link text][reference]

[reference]: https://example.com "Title"
```

Generated events in 0.13.0:
```rust
Event::Start(Tag::Paragraph)
Event::Start(Tag::Link(LinkType::Reference, "https://example.com".into(), "Title".into()))
Event::Text("Link text".into())
Event::End(Tag::Link(LinkType::Reference, "https://example.com".into(), "Title".into()))
Event::End(Tag::Paragraph)

// In 0.13.0, the link reference definition is accessible through the parser API
// but doesn't generate events by default
```

*Key difference from 0.9.3*: In 0.13.0, link reference definitions are part of the AST and accessible through the API, but don't generate events in the main event stream.

### Table Events

For GFM tables:
```markdown
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
```

Generated events in 0.13.0:
```rust
Event::Start(Tag::Table(vec![Alignment::Left, Alignment::Left]))
Event::Start(Tag::TableHead)
Event::Start(Tag::TableRow)
Event::Start(Tag::TableCell)
Event::Text("Header 1".into())
Event::End(Tag::TableCell)
Event::Start(Tag::TableCell)
Event::Text("Header 2".into())
Event::End(Tag::TableCell)
Event::End(Tag::TableRow)
Event::End(Tag::TableHead)
Event::Start(Tag::TableBody)
Event::Start(Tag::TableRow)
Event::Start(Tag::TableCell)
Event::Text("Cell 1".into())
Event::End(Tag::TableCell)
Event::Start(Tag::TableCell)
Event::Text("Cell 2".into())
Event::End(Tag::TableCell)
Event::End(Tag::TableRow)
Event::End(Tag::TableBody)
Event::End(Tag::Table(vec![Alignment::Left, Alignment::Left]))
```

*Difference from 0.9.3*: For tables without a body, 0.13.0 won't generate TableBody events, unlike 0.9.3.

### HTML Content Events

HTML block:
```markdown
<div>
  HTML content
</div>
```

Generated events:
```rust
Event::Html("<div>\n  HTML content\n</div>".into())
```

Inline HTML:
```markdown
This is <span>inline HTML</span>.
```

Generated events:
```rust
Event::Start(Tag::Paragraph)
Event::Text("This is ".into())
Event::Html("<span>".into())
Event::Text("inline HTML".into())
Event::Html("</span>".into())
Event::Text(".".into())
Event::End(Tag::Paragraph)
```

### Code Block Events

Fenced code block:
```markdown
```rust
fn main() {
    println!("Hello");
}
```
```

Generated events in 0.13.0:
```rust
Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced("rust".into())))
Event::Text("fn main() {\n    println!(\"Hello\");\n}\n".into())
Event::End(Tag::CodeBlock(CodeBlockKind::Fenced("rust".into())))
```

*Difference from 0.9.3*: In 0.13.0, the language info handling in code blocks is more robust with better handling of edge cases.

### Break Events

Soft and hard breaks:
```markdown
Line with soft break
Line after soft break

Line with hard break  
Line after hard break
```

Generated events:
```rust
Event::Start(Tag::Paragraph)
Event::Text("Line with soft break".into())
Event::SoftBreak
Event::Text("Line after soft break".into())
Event::End(Tag::Paragraph)

Event::Start(Tag::Paragraph)
Event::Text("Line with hard break".into())
Event::HardBreak
Event::Text("Line after hard break".into())
Event::End(Tag::Paragraph)
```

### Footnote Events

With footnotes enabled:
```markdown
Text with footnote[^1].

[^1]: Footnote content.
```

Generated events in 0.13.0:
```rust
Event::Start(Tag::Paragraph)
Event::Text("Text with footnote".into())
Event::FootnoteReference("1".into())
Event::Text(".".into())
Event::End(Tag::Paragraph)

// The footnote definition generates its own set of events
Event::Start(Tag::FootnoteDefinition("1".into()))
Event::Start(Tag::Paragraph)
Event::Text("Footnote content.".into())
Event::End(Tag::Paragraph)
Event::End(Tag::FootnoteDefinition("1".into()))
```

*New in 0.13.0*: Footnote event handling has been improved for better spec compliance.

### Working with Link Reference Definitions

Example of programmatically accessing link reference definitions in 0.13.0:

```rust
use pulldown_cmark::{Parser, Options};

let markdown = r#"
[link][reference]

[reference]: https://example.com "Title"
"#;

let parser = Parser::new_ext(markdown, Options::empty());

// To access the link reference definitions in 0.13.0:
for (ref_name, (dest, title)) in parser.reference_definitions() {
    println!("Reference: {}", ref_name);
    println!("  Destination: {}", dest);
    println!("  Title: {}", title);
}
```

*Key difference from 0.9.3*: This API is not available in 0.9.3, which doesn't expose link reference definitions directly.

## Special Cases and Edge Cases

### Link Reference Resolution

```markdown
[Link text][reference]

[reference]: https://example.com "Title"
```

*Difference from 0.9.3*: In 0.13.0, link reference definitions are stored in their original case before being normalized, allowing more flexible reference handling.

### Escaping in Tables

```markdown
| Column 1 | Column \| with escaped pipe |
|----------|----------------------------|
| Data 1   | Data 2                     |
```

*Difference from 0.9.3*: In 0.13.0, escaping in tables focuses exclusively on pipe characters; all other escaped characters are passed through to inline parsing.

### Nested Elements

```markdown
> This is a *blockquote with **nested** formatting*
> - And a list
> - Inside it
```

*Difference from 0.9.3*: Version 0.13.0 handles complex nested structures more efficiently and provides better adherence to the spec.

## Processing and Optimization

0.13.0 includes significant performance improvements:
- Parsing is 7-10% faster than 0.9.3
- HTML rendering is approximately twice as fast as 0.9.3
- Improved handling of pathological cases:
  - Fixed quadratic behavior with input like `[\\\\...` (many backslashes)
  - Fixed pathological cases with nested brackets like `[]([]([](...`

## Using pulldown-cmark 0.13.0

### Basic Usage Example

```rust
use pulldown_cmark::{Parser, Options, html};

let markdown_input = "Hello, *world*!";
let mut options = Options::empty();
options.insert(Options::ENABLE_TABLES);
options.insert(Options::ENABLE_FOOTNOTES);
options.insert(Options::ENABLE_STRIKETHROUGH);
options.insert(Options::ENABLE_TASKLISTS);

let parser = Parser::new_ext(markdown_input, options);
let mut html_output = String::new();
html::push_html(&mut html_output, parser);
```

### Working with Link Reference Definitions

```rust
use pulldown_cmark::{Parser, Options};

let markdown = r#"
[link][reference]

[reference]: https://example.com "Title"
"#;

let parser = Parser::new_ext(markdown, Options::empty());

for event in parser {
    // Process events and access link reference definitions
    // through the inline parser context
}
```

## Conclusion

pulldown-cmark 0.13.0 represents a significant improvement over 0.9.3 in terms of spec compliance, performance, and API capabilities. The introduction of link reference definition nodes in the AST and improvements to table parsing bring the library closer to full GitHub Flavored Markdown compatibility.

When upgrading from 0.9.3 to 0.13.0, developers should be aware of the changes in link reference handling and table parsing behavior, which may require adjustments to existing code that relies on the specific behavior of earlier versions.
