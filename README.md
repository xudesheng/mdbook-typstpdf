# mdbook-typstpdf

A limited-purpose tool for converting mdbook projects to PDF using Typst, designed for personal use.

## Overview

This tool was created to meet my specific requirements for converting mdbook projects to PDF format. It is not intended as a full-featured or production-ready open-source tool, but rather as a personal utility with limited functionality.

## Installation

To use mdbook-typstpdf, you'll need to install three components:

1. **Install mdBook**

   ```bash
   cargo install mdbook
   ```

2. **Install mdbook-typstpdf**

   ```bash
   cargo install mdbook-typstpdf --locked
   ```

3. **Install Typst CLI**

   Follow the installation instructions from the [Typst CLI repository](https://github.com/typst/typst) or download from the [Typst releases page](https://github.com/typst/typst/releases).

   Make sure the `typst` command is available in your PATH.

## Workflow

The current workflow is as follows:

1. Create an mdbook project
2. Write each chapter using markdown
3. Use `mdbook build` to publish content to the `book` directory
   - HTML format publishing is an out-of-the-box feature of mdbook
4. PDF publishing consists of the following steps:
   - Parse the `SUMMARY.md` file to locate each chapter's markdown content (note: currently ignores any additional content in `SUMMARY.md` and only uses it to identify chapter files)
   - Convert each chapter's markdown file to a Typst file
   - Combine all chapter Typst files into a book-level Typst file
   - If a Typst template is defined, reference it in the book-level Typst file
   - Use the Typst CLI to convert the book-level Typst file to PDF format
   - Intermediate Typst files for individual chapters can be either preserved or removed

## Current Limitations

1. The book's index is not based on the structure defined in `SUMMARY.md`, but is instead determined by the Typst template
2. Code blocks only support out-of-the-box functionality; extensions (particularly diagram-generating extensions) are not supported
3. MathJax is not supported

## Status

This is not a serious open-source tool but rather a utility designed to meet my current conversion needs. API documentation is minimal or non-existent. If there is sufficient interest, more documentation may be added in the future.

If you are interested in this tool and would like to add support for new format conversions, please feel free to open an issue.

**Note:** This is a personal tool created in my spare time for temporary use only, using it at your risk.

**Note:** Credit to: https://github.com/max-heller/mdbook-pandoc . I was trying to use it but I couldn't figure out how to control the layout via a template, so I start to build this one for me to use temporarly.

