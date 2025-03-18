#let best_practice_template(
  doc_title: "Document Title",
  doc_version: "1.0",
  abstract: "Document abstract",
  doc_author: "Author Name",
  author_email: "author@example.com",
  doc_date: "January 1, 2023",
  software_tested: "Software v1.0",
  feedback_email: "feedback@example.com",
  reviewers: "Reviewer Names",
  body
) = {
  // control how the code block is displayed.
  show raw: it => {
    if it.block {
      // multiline code block style (using triple or more backticks)
      block(
        width: 100%,
        fill: rgb("#f6f8fa"),
        radius: 4pt,
        stroke: (left: 2pt + rgb("#e1e4e8")),
        inset: (x: 5pt, left: 8pt),
        it
      )
    } else {
      // inline code block style (using single backtick)
      box(
        fill: rgb("#f5f5f5"),
        inset: (x: 3pt, y: 0pt),
        radius: 2pt,
        it
      )
    }
  }

  // Center all images
  show image: it => align(center, it)

  // Center all tables
  show table: it => align(center, it)
  
  // Always show figure index for all figures
  show figure: it => {
    align(center)[
      #it.body
      #text(weight: "bold")[Figure #counter(figure).display()]
    ]
  }

  // Helper function to get template images with hardcoded paths
  let get_template_image(name, width) = {
    // During development, use placeholders
    // rect(width: width, height: width * 0.6, fill: rgb("f1f3f4"))
    
    // When images are available, uncomment this:
    image("images/" + name, width: width)
  }

  // Page setup
  set page(
    margin: (top: 120pt, bottom: 120pt, left: 50pt, right: 50pt),
    header: {
      grid(
        columns: (1fr, 1fr),
        
      )
      
      // Only show line on pages other than the cover page
      context {
        if counter(page).at(here()).first() != 1 {
          v(5pt)
          line(length: 100%)
        }
      }
    },
    background: context {
      if counter(page).at(here()).first() == 1 {
        place(
          top + right,
          dx: 30pt,
          dy: -30pt,
          get_template_image("background.png", 420pt)
        )
      }
    },
    footer: context {
      if counter(page).at(here()).first() != 1 {
        // Add horizontal line above footer with no space below
        line(length: 100%, stroke: 0.5pt)
        
        // Combined footer table with better vertical spacing
        table(
          columns: (2fr, 0.5fr, 1.5fr, 1.5fr),
          rows: (auto, auto),
          inset: (x: 5pt, y: 6pt), // Increased vertical padding between content and borders
          stroke: 0.5pt,
          
          // Row 1
          [
            #block(spacing: 0pt)[
              #text(size: 10pt, weight: "bold")[Best Practice Name]
              #linebreak()
              #text(size: 9pt)[#doc_title]
            ]
          ], 
          table.cell(align: center)[
            #block(spacing: 0pt)[
              #text(size: 9pt, weight: "bold")[Version]
              #linebreak()
              #text(size: 9pt)[#doc_version]
            ]
          ], 
          [
            #block(spacing: 0pt)[
              #text(size: 10pt, weight: "bold")[Version Date]
              #linebreak()
              #text(size: 9pt)[#doc_date]
            ]
          ], 
          [
            #block(spacing: 0pt)[
              #text(size: 10pt, weight: "bold")[Author]
              #linebreak()
              #text(size: 9pt)[#doc_author]
            ]
          ],
          
          // Row 2
          table.cell([
            #block(spacing: 0pt)[
              #text(size: 10pt, weight: "bold")[Software Versions Tested]
              #linebreak()
              #text(size: 9pt)[#software_tested]
            ]
          ]),
          table.cell(align: center)[
            #block(spacing: 0pt)[
              #align(center)[
                #text(size: 9pt, weight: "bold")[Page]
                #linebreak()
                #text(size: 9pt)[#counter(page).display()]
              ]
            ]
          ],
          [
            #block(spacing: 0pt)[
              #text(size: 9pt, weight: "bold")[Please Send Feedback to:]
              #linebreak()
              #text(size: 9pt)[#feedback_email]
            ]
          ], 
          [
            #block(spacing: 0pt)[
              #text(size: 10pt, weight: "bold")[Reviewer]
              #linebreak()
              #text(size: 8pt)[#reviewers]
            ]
          ],
        )
        
        v(0.1pt)
      }
    }
  )

  // Configure headings
  set heading(numbering: "1.1")

  // Only make level 1 headings (chapters) start on a new page
  show heading.where(level: 1): it => {
    pagebreak(weak: true)
    it
  }

  // Cover page with custom layout
  block(height: 100%)[
    // Title-version block starts at 50% of page height
    #v(50%)
    
    #align(right)[
      // Title-version block (fixed spacing)
      #block(width: 100%)[
        #align(right)[
          #text(size: 28pt, weight: "bold")[#doc_title]
          #linebreak()
          #text(size: 12pt)[Version #doc_version]
        ]
      ]
      
      // Moderate space between title-version and abstract
      #v(50pt)
      
      // Abstract block (fixed spacing)
      #block(width: 100%)[
        #align(right)[
          #text(size: 20pt, weight: "bold")[Abstract]
          #linebreak()
          #block(width: 80%)[
            #align(right)[
              #text(size: 12pt)[#abstract]
            ]
          ]
        ]
      ]
      
      // Flexible space to push author block to bottom
      #v(1fr)
      
      // Author block (fixed spacing) at bottom
      #block(width: 100%)[
        #align(right)[
          #text(size: 20pt)[#doc_author]
          #linebreak()
          #text(size: 12pt)[#author_email]
        ]
      ]
      
      // Small space to position author block at top of footer
      #v(10pt)
    ]
  ]

  pagebreak()
  
  // Table of Contents page
  align(center)[
    #text(size: 24pt, weight: "bold")[Table of Contents]
  ]
  
  v(20pt)
  
  // Configure the outline to only show level 1 and 2 headings
  outline(
    title: none,
    indent: 1em,
    depth: 2
  )
  
  pagebreak()
  
  // Main content
  body
} 