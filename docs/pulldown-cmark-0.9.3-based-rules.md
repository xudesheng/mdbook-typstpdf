# Markdown to Typst Conversion Rules

This document outlines the rules used to convert Markdown to Typst format using pulldown-cmark 0.9.3 in the mdbook-typstpdf project. These rules can be referenced when upgrading to newer versions of pulldown-cmark.

## Parser Configuration

```rust
let mut options = Options::empty();
options.insert(Options::ENABLE_TABLES);
options.insert(Options::ENABLE_FOOTNOTES);
options.insert(Options::ENABLE_STRIKETHROUGH);
options.insert(Options::ENABLE_TASKLISTS);

let parser = Parser::new_ext(content, options);
```

## Basic Element Conversions

### Headings

- **Markdown**: `# Heading Level 1`, `## Heading Level 2`, etc.
- **Typst**: `= Heading Level 1`, `== Heading Level 2`, etc.
- **Implementation**: 
  ```rust
  Tag::Heading(level, _, _) => {
      typst_output.push_str("\n\n");
      typst_output.push_str(&format!("{} ", "=".repeat(level as usize)));
  }
  ```

### Paragraphs

- **Markdown**: Plain text separated by blank lines
- **Typst**: Text separated by newlines
- **Implementation**:
  ```rust
  Tag::Paragraph => {
      if table_state != TableState::None {
          // Inside a table, don't add paragraph markers
      } else {
          typst_output.push('\n');
      }
  }
  ```

### Emphasis (Italic)

- **Markdown**: `*italic*` or `_italic_`
- **Typst**: `_italic_`
- **Implementation**:
  ```rust
  Tag::Emphasis => {
      typst_output.push('_');
  }
  // ... and at the end:
  Tag::Emphasis => {
      typst_output.push('_');
  }
  ```

### Strong (Bold)

- **Markdown**: `**bold**` or `__bold__`
- **Typst**: `*bold*`
- **Implementation**:
  ```rust
  Tag::Strong => {
      typst_output.push('*');
  }
  // ... and at the end:
  Tag::Strong => {
      typst_output.push('*');
      typst_output.push(' '); // Extra space added after bold
  }
  ```

### Strikethrough

- **Markdown**: `~~strikethrough~~`
- **Typst**: `#strike[strikethrough]`
- **Implementation**:
  ```rust
  Tag::Strikethrough => {
      typst_output.push_str("#strike[");
  }
  // ... and at the end:
  Tag::Strikethrough => {
      typst_output.push(']');
  }
  ```

### Links

- **Markdown**: `[link text](url)`
- **Typst**: `#link("url")[link text]`
- **Implementation**:
  ```rust
  Tag::Link(_, url, _) => {
      typst_output.push_str(&format!("#link(\"{}\")[", url));
  }
  // ... and at the end:
  Tag::Link(_, _, _) => {
      typst_output.push(']');
  }
  ```

### Images

- **Markdown**: `![alt text](path/to/image)`
- **Typst**: `#figure(image("path/to/image", width: [calculated]), caption: [])`
- **Implementation**:
  ```rust
  Tag::Image(_, url, title) => {
      let image_path = url.to_string();
      
      if !image_path.starts_with("http://") && !image_path.starts_with("https://") {
          let (width, _height) = calculate_image_size(&image_path, ctx);
          typst_output.push_str(&format!("#figure(\n  image(\"{}\", width: {}),\n  caption: []\n)", 
              image_path, width
          ));
      } else {
          typst_output.push_str(&format!("#image(\"{}\", alt: \"{}\")", url, title));
      }
  }
  ```

### Block Quotes

- **Markdown**: `> quoted text`
- **Typst**: `#quote[quoted text]`
- **Implementation**:
  ```rust
  Tag::BlockQuote => {
      typst_output.push_str("\n#quote[\n");
  }
  // ... and at the end:
  Tag::BlockQuote => {
      typst_output.push_str("\n]\n");
  }
  ```

### Code Blocks

- **Markdown**: 
  ````
  ```language
  code
  ```
  ````
- **Typst**: `#raw(lang: "language", "code")`
- **Implementation**:
  ```rust
  Tag::CodeBlock(kind) => {
      if need_code_block_template {
          typst_output.push_str("#code_block[\n");
      }
      typst_output.push_str("#raw(");
      if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
          if !lang.is_empty() {
              typst_output.push_str(&format!("lang: \"{}\", ", lang));
          }
      }
      typst_output.push('"');
  }
  // ... and at the end:
  Tag::CodeBlock(_) => {
      typst_output.push_str("\")");
      if need_code_block_template {
          typst_output.push_str("\n]");
      }
      typst_output.push('\n');
  }
  ```

### Inline Code

- **Markdown**: `` `code` ``
- **Typst**: `` `code` ``
- **Implementation**:
  ```rust
  Event::Code(code) => {
      typst_output.push_str(&format!("`{}`", code));
  }
  ```

### Lists

- **Markdown**:
  ```
  - Item 1
  - Item 2
    - Nested Item
  ```
- **Typst**:
  ```
  - Item 1
  - Item 2
    - Nested Item
  ```
- **Implementation**:
  ```rust
  Tag::List(first_item_number) => {
      if !typst_output.ends_with('\n') {
          typst_output.push('\n');
      }
      if first_item_number.is_some() {
          list_stack.push(ListType::Ordered(()));
      } else {
          list_stack.push(ListType::Unordered);
      }
  }
  
  Tag::Item => {
      if let Some(list_type) = list_stack.last() {
          match list_type {
              ListType::Unordered => {
                  let indent = if list_stack.len() > 1 {
                      " ".repeat((list_stack.len() - 1) * 2)
                  } else {
                      "".to_string()
                  };
                  typst_output.push_str(&format!("{}- ", indent));
              }
              ListType::Ordered(_) => {
                  let indent = if list_stack.len() > 1 {
                      " ".repeat((list_stack.len() - 1) * 2)
                  } else {
                      "".to_string()
                  };
                  typst_output.push_str(&format!("{}+ ", indent));
              }
          }
      }
  }
  ```

### Tables

- **Markdown**:
  ```
  | Header 1 | Header 2 |
  |----------|----------|
  | Cell 1   | Cell 2   |
  ```
- **Typst**:
  ```
  #table(
    columns: 2,
    table.header[Header 1][Header 2],
    [Cell 1], [Cell 2]
  )
  ```
- **Implementation**:
  ```rust
  Tag::Table(alignments) => {
      table_state = TableState::InTable;
      _in_table = true;
      typst_output.push_str("#table(\n");
      table_columns = alignments.len();
      typst_output.push_str(&format!("  columns: {},\n", table_columns));
  }
  
  Tag::TableHead => {
      table_state = TableState::InHeader;
      in_table_head = true;
      typst_output.push_str("  table.header");
  }
  
  Tag::TableRow => {
      table_state = TableState::InRow;
      _in_row = true;
  }
  
  Tag::TableCell => {
      typst_output.push('[');
  }
  ```

### Footnotes

- **Markdown**: `Here is a footnote reference[^1]`
- **Typst**: `Here is a footnote reference#footnote[1]`
- **Implementation**:
  ```rust
  Tag::FootnoteDefinition(ref _footnote_id) => {
      typst_output.push_str("#footnote[");
  }
  // ... and at the end:
  Tag::FootnoteDefinition(_) => {
      typst_output.push(']');
  }
  ```

## Special Case Handling

### HTML Tags

HTML tags are mostly ignored, except for `<img>` tags which are converted to Typst `#image` or `#figure` commands:

```rust
Event::Html(html) => {
    let html_str = html.to_string();
    if html_str.contains("<img") {
        if let Some(cap) = RE_HTML_IMG.captures(&html_str) {
            if let Some(src) = cap.get(1) {
                let image_path = src.as_str();
                
                if !image_path.starts_with("http://") && !image_path.starts_with("https://") {
                    let (width, _height) = calculate_image_size(image_path, ctx);
                    typst_output.push_str(&format!("#figure(\n  image(\"{}\", width: {}),\n  caption: []\n)", 
                        image_path, width
                    ));
                } else {
                    typst_output.push_str(&format!("#image(\"{}\", alt: \"\")", image_path));
                }
            }
        }
    }
    // Ignore other HTML tags
}
```

### Escaping Special Characters

Typst has its own special characters that need to be escaped when coming from Markdown:

```rust
fn escape_typst_special_chars(text: &str) -> String {
    // Skip leading # characters that might be from Markdown headings
    // Special case for "unquoted *" pattern in command line examples
    // Special handling for URLs to avoid escaping asterisks in URLs
    
    // Characters that need escaping: # * _ ` $ { } [ ]
    for c in text_to_process.chars() {
        match c {
            '#' | '*' | '_' | '`' | '$' | '{' | '}' | '[' | ']' => {
                result.push('\\');
                result.push(c);
            }
            '\\' => {
                // Double backslashes in string literals
                result.push('\\');
                result.push('\\');
            }
            '"' => {
                // Escape quotes in string literals
                result.push('\\');
                result.push('"');
            }
            _ => result.push(c),
        }
    }
}
```

### Handling of $ Variables

Dollar signs followed by variable names are escaped to prevent Typst interpreting them as math expressions:

```rust
fn escape_dollar_signs(text: &str) -> String {
    let re = regex::Regex::new(r"\$([A-Za-z][A-Za-z0-9_]*)").unwrap();
    
    // Logic to add backslash before $ if not already escaped
    // ...
}
```

## Post-Processing

After initial conversion, several regex-based replacements are applied to fix common issues:

1. List formatting fixes:
   ```rust
   // Remove newlines after list item markers
   let list_item_re = regex::Regex::new(r"- \n").unwrap();
   result = list_item_re.replace_all(&processed, "- ").to_string();
   ```

2. Figure text closure:
   ```rust
   // Fix text directly following a figure
   let figure_text_re = regex::Regex::new(r"\)\n([^#\n][^\n]+)").unwrap();
   result = figure_text_re.replace_all(&result, ")\n#[\n$1\n]\n").to_string();
   ```

3. URL asterisk handling:
   ```rust
   // Fix asterisks in URLs in content blocks
   let url_asterisk_re = regex::Regex::new(r"#\[\n([^]]*http[^]]*\*[^]]*)\n\]").unwrap();
   result = url_asterisk_re.replace_all(&result, |caps: &regex::Captures| {
       let content = &caps[1];
       let fixed_content = content.replace("*", "\\*");
       format!("#[\n{}\n]", fixed_content)
   }).to_string();
   ```

4. Nested figure handling:
   ```rust
   // Fix nested figures in content blocks
   let nested_figure_re = regex::Regex::new(r"#\[\n([^]]*)(#figure\()").unwrap();
   result = nested_figure_re.replace_all(&result, |caps: &regex::Captures| {
       let content_before = &caps[1];
       let figure_start = &caps[2];
       format!("#[\n{}\n]\n{}", content_before.trim(), figure_start)
   }).to_string();
   ```

5. Formatting fixes:
   ```rust
   // Fix spacing issues in bold text
   result = result.replace("*:", "* :");
   result = result.replace("*is", "* is");
   result = result.replace("*pod", "* pod");
   ```

## Image Sizing

Images are dynamically sized based on their dimensions and the page layout:

```rust
fn calculate_image_size(image_path: &str, ctx: &mdbook::renderer::RenderContext) -> (String, String) {
    // Default values
    let default_width = "100%";
    let default_height = "auto";
    
    // Try to get image dimensions and apply sizing rules:
    // 1. If height > 45% of page height: scale down
    // 2. If width > 95% of page width but height < 45%: scale to 95% width
    // 3. Default: use 100% width
}
```
