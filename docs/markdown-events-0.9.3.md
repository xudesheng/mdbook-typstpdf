# Markdown to pulldown-cmark 0.9.3 Events Mapping

This document maps Markdown content to the events triggered when parsed by pulldown-cmark 0.9.3 in Rust. Understanding these events is essential for working with the parsed content programmatically.

## Basic Structure

In pulldown-cmark, parsing generates a sequence of `Event` enum variants. The main event types are:

- `Start(Tag)` - Marks the beginning of a block or inline element
- `End(Tag)` - Marks the end of a block or inline element
- `Text(String)` - Plain text content
- `Code(String, Option<String>)` - Inline code with optional language info
- `Html(String)` - HTML block or inline content
- `FootnoteReference(String)` - Reference to a footnote
- `SoftBreak` - Soft line break (newline in source that doesn't create a new paragraph)
- `HardBreak` - Hard line break (forced newline)
- `Rule` - Horizontal rule/thematic break
- `TaskListMarker(bool)` - Task list marker (checked or unchecked)

## Element Mappings

### Headings

```markdown
# Heading 1
## Heading 2
```

Events:
```rust
Start(Tag::Heading(HeadingLevel::H1, None, Vec::new()))
Text("Heading 1")
End(Tag::Heading(HeadingLevel::H1, None, Vec::new()))

Start(Tag::Heading(HeadingLevel::H2, None, Vec::new()))
Text("Heading 2")
End(Tag::Heading(HeadingLevel::H2, None, Vec::new()))
```

Setext headings (underlined):
```markdown
Heading 1
=========
```

Events:
```rust
Start(Tag::Heading(HeadingLevel::H1, None, Vec::new()))
Text("Heading 1")
End(Tag::Heading(HeadingLevel::H1, None, Vec::new()))
```

### Paragraphs

```markdown
This is a paragraph.

This is another paragraph.
```

Events:
```rust
Start(Tag::Paragraph)
Text("This is a paragraph.")
End(Tag::Paragraph)

Start(Tag::Paragraph)
Text("This is another paragraph.")
End(Tag::Paragraph)
```

### Line Breaks

Soft break (single newline):
```markdown
Line one
Line two
```

Events:
```rust
Start(Tag::Paragraph)
Text("Line one")
SoftBreak
Text("Line two")
End(Tag::Paragraph)
```

Hard break (backslash or two spaces at end of line):
```markdown
Line one  
Line two

Line one\
Line two
```

Events:
```rust
Start(Tag::Paragraph)
Text("Line one")
HardBreak
Text("Line two")
End(Tag::Paragraph)

Start(Tag::Paragraph)
Text("Line one")
HardBreak
Text("Line two")
End(Tag::Paragraph)
```

### Emphasis and Strong

```markdown
*Italic* or _Italic_
**Bold** or __Bold__
***Bold and italic***
```

Events:
```rust
Start(Tag::Paragraph)
Start(Tag::Emphasis)
Text("Italic")
End(Tag::Emphasis)
Text(" or ")
Start(Tag::Emphasis)
Text("Italic")
End(Tag::Emphasis)
End(Tag::Paragraph)

Start(Tag::Paragraph)
Start(Tag::Strong)
Text("Bold")
End(Tag::Strong)
Text(" or ")
Start(Tag::Strong)
Text("Bold")
End(Tag::Strong)
End(Tag::Paragraph)

Start(Tag::Paragraph)
Start(Tag::Strong)
Start(Tag::Emphasis)
Text("Bold and italic")
End(Tag::Emphasis)
End(Tag::Strong)
End(Tag::Paragraph)
```

### Strikethrough (GFM Extension)

```markdown
~~Strikethrough text~~
```

Events:
```rust
Start(Tag::Paragraph)
Start(Tag::Strikethrough)
Text("Strikethrough text")
End(Tag::Strikethrough)
End(Tag::Paragraph)
```

### Code

Inline code:
```markdown
`inline code`
```

Events:
```rust
Start(Tag::Paragraph)
Code("inline code", None)
End(Tag::Paragraph)
```

Code block with backticks:
```markdown
```
code block
```
```

Events:
```rust
Start(Tag::CodeBlock(CodeBlockKind::Fenced(String::new())))
Text("code block")
End(Tag::CodeBlock(CodeBlockKind::Fenced(String::new())))
```

Code block with language:
```markdown
```rust
fn main() {}
```
```

Events:
```rust
Start(Tag::CodeBlock(CodeBlockKind::Fenced("rust".into())))
Text("fn main() {}")
End(Tag::CodeBlock(CodeBlockKind::Fenced("rust".into())))
```

Indented code block:
```markdown
    indented code
```

Events:
```rust
Start(Tag::CodeBlock(CodeBlockKind::Indented))
Text("indented code")
End(Tag::CodeBlock(CodeBlockKind::Indented))
```

### Lists

Unordered list:
```markdown
- Item 1
- Item 2
  - Nested item
```

Events:
```rust
Start(Tag::List(None))
Start(Tag::Item)
Start(Tag::Paragraph)
Text("Item 1")
End(Tag::Paragraph)
End(Tag::Item)
Start(Tag::Item)
Start(Tag::Paragraph)
Text("Item 2")
End(Tag::Paragraph)
Start(Tag::List(None))
Start(Tag::Item)
Start(Tag::Paragraph)
Text("Nested item")
End(Tag::Paragraph)
End(Tag::Item)
End(Tag::List(None))
End(Tag::Item)
End(Tag::List(None))
```

Ordered list:
```markdown
1. Item 1
2. Item 2
```

Events:
```rust
Start(Tag::List(Some(1))) // The number indicates starting number
Start(Tag::Item)
Start(Tag::Paragraph)
Text("Item 1")
End(Tag::Paragraph)
End(Tag::Item)
Start(Tag::Item)
Start(Tag::Paragraph)
Text("Item 2")
End(Tag::Paragraph)
End(Tag::Item)
End(Tag::List(Some(1)))
```

### Task Lists (GFM Extension)

```markdown
- [ ] Unchecked task
- [x] Checked task
```

Events:
```rust
Start(Tag::List(None))
Start(Tag::Item)
TaskListMarker(false)
Start(Tag::Paragraph)
Text("Unchecked task")
End(Tag::Paragraph)
End(Tag::Item)
Start(Tag::Item)
TaskListMarker(true)
Start(Tag::Paragraph)
Text("Checked task")
End(Tag::Paragraph)
End(Tag::Item)
End(Tag::List(None))
```

### Links

Inline link:
```markdown
[Link text](https://example.com "Title")
```

Events:
```rust
Start(Tag::Paragraph)
Start(Tag::Link(LinkType::Inline, "https://example.com".into(), "Title".into()))
Text("Link text")
End(Tag::Link(LinkType::Inline, "https://example.com".into(), "Title".into()))
End(Tag::Paragraph)
```

Reference link:
```markdown
[Link text][reference]

[reference]: https://example.com "Title"
```

Events:
```rust
Start(Tag::Paragraph)
Start(Tag::Link(LinkType::Reference, "https://example.com".into(), "Title".into()))
Text("Link text")
End(Tag::Link(LinkType::Reference, "https://example.com".into(), "Title".into()))
End(Tag::Paragraph)
```

Autolink:
```markdown
<https://example.com>
```

Events:
```rust
Start(Tag::Paragraph)
Start(Tag::Link(LinkType::Autolink, "https://example.com".into(), "".into()))
Text("https://example.com")
End(Tag::Link(LinkType::Autolink, "https://example.com".into(), "".into()))
End(Tag::Paragraph)
```

### Images

```markdown
![Alt text](image.jpg "Title")
```

Events:
```rust
Start(Tag::Paragraph)
Start(Tag::Image(LinkType::Inline, "image.jpg".into(), "Title".into()))
Text("Alt text")
End(Tag::Image(LinkType::Inline, "image.jpg".into(), "Title".into()))
End(Tag::Paragraph)
```

### Blockquotes

```markdown
> Blockquote text
> More text
```

Events:
```rust
Start(Tag::BlockQuote)
Start(Tag::Paragraph)
Text("Blockquote text")
SoftBreak
Text("More text")
End(Tag::Paragraph)
End(Tag::BlockQuote)
```

Nested blockquotes:
```markdown
> Outer quote
>> Nested quote
```

Events:
```rust
Start(Tag::BlockQuote)
Start(Tag::Paragraph)
Text("Outer quote")
End(Tag::Paragraph)
Start(Tag::BlockQuote)
Start(Tag::Paragraph)
Text("Nested quote")
End(Tag::Paragraph)
End(Tag::BlockQuote)
End(Tag::BlockQuote)
```

### Horizontal Rules

```markdown
---

***

___
```

Each produces:
```rust
Rule
```

### Tables (GFM Extension)

```markdown
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
```

Events:
```rust
Start(Tag::Table(vec[Alignment::Left, Alignment::Left]))
Start(Tag::TableHead)
Start(Tag::TableRow)
Start(Tag::TableCell)
Text("Header 1")
End(Tag::TableCell)
Start(Tag::TableCell)
Text("Header 2")
End(Tag::TableCell)
End(Tag::TableRow)
End(Tag::TableHead)
Start(Tag::TableBody)
Start(Tag::TableRow)
Start(Tag::TableCell)
Text("Cell 1")
End(Tag::TableCell)
Start(Tag::TableCell)
Text("Cell 2")
End(Tag::TableCell)
End(Tag::TableRow)
End(Tag::TableBody)
End(Tag::Table(vec[Alignment::Left, Alignment::Left]))
```

### HTML Content

Inline HTML:
```markdown
This is <span>inline HTML</span>.
```

Events:
```rust
Start(Tag::Paragraph)
Text("This is ")
Html("<span>")
Text("inline HTML")
Html("</span>")
Text(".")
End(Tag::Paragraph)
```

HTML Block:
```markdown
<div>
  HTML block content
</div>
```

Events:
```rust
Html("<div>\n  HTML block content\n</div>")
```

### Footnotes (Extension)

```markdown
Text with a footnote[^1].

[^1]: Footnote content.
```

Events:
```rust
Start(Tag::Paragraph)
Text("Text with a footnote")
FootnoteReference("1")
Text(".")
End(Tag::Paragraph)

// Note: The footnote definition itself generates more complex events
// including a footnote definition tag
```

## Escaped Characters

```markdown
\*Not italic\*
```

Events:
```rust
Start(Tag::Paragraph)
Text("*Not italic*")
End(Tag::Paragraph)
```

## Special Cases and Edge Cases

### Empty Document

```markdown
```

Events:
```rust
// No events
```

### Multiple Adjacent Elements

```markdown
# Heading
Paragraph
- List item
```

Events: All relevant events for each element in sequence.

### Comments

```markdown
<!-- This is a comment -->
```

Events:
```rust
Html("<!-- This is a comment -->")
```

## Understanding Event Flow

When working with pulldown-cmark events:

1. Each block element has a `Start` and `End` event
2. Block elements can contain other block or inline elements
3. Inline elements appear within block elements and also have `Start` and `End` events
4. Text content appears as `Text` events
5. Some special elements like code spans, HTML, and horizontal rules have their own specialized events
6. The events are emitted in document order as they are encountered

By mapping the Markdown source to its corresponding events, you can better understand how pulldown-cmark processes a document and design your event handling code accordingly.
