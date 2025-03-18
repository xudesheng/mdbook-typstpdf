use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, CodeBlockKind};
use std::{cmp::PartialEq, path::{Path, PathBuf}};
use regex;
use lazy_static::lazy_static;
use imagesize;
use std::fs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::blocking::Client;


use crate::config::IMAGE_DIR;

use super::Config;

// Make TableState implement PartialEq
#[derive(Debug, Clone, Copy, PartialEq)]
enum TableState {
    None,
    InTable,
    InHeader,
    InRow,
}

// Helper enum to track list types
enum ListType {
    Ordered(Option<u64>),
    Unordered,
}

lazy_static! {
    static ref RE_HTML_IMG: regex::Regex = regex::Regex::new(r#"<img[^>]*src=["']([^"']+)["']"#).unwrap();
}

// 页面尺寸常量 (A4 paper in points)
const PAGE_WIDTH: f64 = 595.0;  // A4 width in points
const PAGE_HEIGHT: f64 = 842.0; // A4 height in points
// const MAX_WIDTH_PERCENT: f64 = 0.95; // 95% of page width
// const MAX_HEIGHT_PERCENT: f64 = 0.45; // 45% of page height

// 计算图片的合适尺寸
fn calculate_image_size(image_path: &str, 
    max_width_percent: &Option<f64>, // for example, 95% of the page width
    max_height_percent: &Option<f64>, // for example, 45% of page height
    ctx: &mdbook::renderer::RenderContext
) -> (String, String) {
    // 默认值，如果无法获取图片尺寸
    let default_width = "100%";
    let default_height = "auto";

    if max_width_percent.is_none() && max_height_percent.is_none(){
        // return with default value if both are none
        return (default_width.to_string(), default_height.to_string());
    }

    let mut max_width_percent = match max_width_percent{
        Some(num)=>*num,
        None=> 1.0
    };
    let mut max_height_percent = match max_height_percent{
        Some(num) => *num,
        None => 1.0
    };
    if max_width_percent>1.0 || max_width_percent<=0.0 {
        log::error!("Image max width percent is out of range:{}, change it to 1.0 or 100%",max_width_percent);
        max_width_percent = 1.0;
    }
    if max_height_percent>1.0 || max_height_percent<=0.0{
        log::error!("Image max height percent is out of range:{}, change it to 1.0 or 100%",max_height_percent);
        max_height_percent = 1.0;
    }
    
    // 获取图片的完整路径
    let src_dir = ctx.root.join(&ctx.config.book.src);
    let full_path = src_dir.join(image_path);
    
    // 尝试获取图片尺寸
    match imagesize::size(full_path) {
        Ok(size) => {
            let img_width = size.width as f64;
            let img_height = size.height as f64;
            
            // 计算图片在100%宽度时的高度比例
            let height_ratio = img_height / img_width;
            let full_width_height = PAGE_WIDTH * height_ratio;
            
            // 检查规则1：如果高度超过页面高度的45%
            if full_width_height > PAGE_HEIGHT * max_height_percent {
                // 需要缩小图片
                let max_height = PAGE_HEIGHT * max_height_percent;
                let new_width = max_height / height_ratio;
                let width_percent = (new_width / PAGE_WIDTH) * 100.0;
                
                return (format!("{}%", width_percent.round()), default_height.to_string());
            }
            
            // 检查规则2：如果高度低于45%但宽度超过95%
            if img_width > PAGE_WIDTH * max_width_percent && full_width_height < PAGE_HEIGHT * max_height_percent {
                return (format!("{}%", (max_width_percent * 100.0).round()), default_height.to_string());
            }
            
            // 默认使用100%宽度
            (default_width.to_string(), default_height.to_string())
        },
        Err(_) => {
            // 如果无法获取图片尺寸，使用默认值
            (default_width.to_string(), default_height.to_string())
        }
    }
}

impl Config {
    pub fn parse_chapter_content(
        &self, 
        chapter: &mdbook::book::Chapter,
        content: &str, 
        dst_file_path: &std::path::Path,
        image_parent_dir: &std::path::Path,
        ctx: &mdbook::renderer::RenderContext
    ) -> anyhow::Result<String> {
        // Parse the chapter content from markdown to typst format
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);

        // preprocess the <img> tag, example: <img src="docs/01-introduction/image-20250224001420194.png" alt="image-20250224001420194" style="zoom:50%;" />
        // to: ![image-20250224001420194](docs/01-introduction/image-20250224001420194.png)
        let content = preprocess_img_tag(content);

        let parser = Parser::new_ext(&content, options);
        let mut typst_output = String::new();

        let image_folder_name = IMAGE_DIR;
        let image_dir = image_parent_dir.join(image_folder_name);
        
        // Create the image directory if it doesn't exist
        if !image_dir.exists() {
            std::fs::create_dir_all(&image_dir)?;
            log::debug!("Created image directory at {:?}", image_dir);
        }

        

        // Calculate the correct relative path to templates
        let template_rel_path = self.calculate_relative_path_to_templates(chapter,ctx);
        log::debug!("Template relative path for {}: {}", dst_file_path.display(), template_rel_path);
        // Add quote block setting at the beginning of the document
        if let Some(chapter_imports) = &self.chapter_imports {
            typst_output.push_str(chapter_imports);
        }
        // typst_output.push_str("#import \"@preview/gentle-clues:0.6.0\": *\n\n");

        let mut list_stack = Vec::new();
        let mut table_state = TableState::None;
        
        
        
        let mut table_columns:usize = 0;
        
        // Track current image caption status
        // let mut current_image_has_caption = false;
        let mut in_image = false;
        
        // Add a tracking variable at the beginning of your parse_chapter_content function
        let mut in_strong_context = false;
        let mut handled_bold_url = false;

        let mut in_code_block = false;
        let mut is_fenced_code_block = false;
        let mut code_block_language = None;
        let mut first_para_in_list_item = false; // there may be multiple paras inisde a list item.
        
        for event in parser {
            log::trace!("event:{:?}",event);
            match event {
                Event::Start(tag) => match tag {
                    Tag::Paragraph => {
                        if table_state != TableState::None {
                            // Inside a table, don't add paragraph markers
                        } else if !list_stack.is_empty(){
                            // inside a list, need to handle ident differently.
                            if first_para_in_list_item {
                                first_para_in_list_item = false;
                            }else{
                                // ensure new line and ident
                                let item_ident = "  ".repeat(list_stack.len());
                                typst_output.push_str(&format!("\n{}",item_ident));
                            }
                        } else {
                            typst_output.push('\n');
                        }
                    }
                    Tag::Heading { level, .. } => {
                        typst_output.push_str("\n\n");
                        typst_output.push_str(&format!("{} ", "=".repeat(level as usize)));
                    }
                    Tag::BlockQuote(_) => {
                        // Ensure a clean start for the blockquote
                        if !typst_output.is_empty() && !typst_output.ends_with('\n') {
                            typst_output.push('\n');
                        }
                        typst_output.push_str("#quote[");
                    }
                    Tag::CodeBlock(kind) => {
                        log::debug!("chapter: {:?}, Code block kind: {:?}", chapter.name, kind);
                        in_code_block = true;
                        if let CodeBlockKind::Fenced(lang) = kind {
                            is_fenced_code_block = true;
                            if !lang.is_empty() {
                                code_block_language = Some(lang.to_string());
                            }
                        }
                        
                    }
                    Tag::List(start) => {
                        // Ensure we start on a new line
                        if !typst_output.ends_with('\n') {
                            typst_output.push('\n');
                        }
                        
                        if start.is_some() {
                            list_stack.push(ListType::Ordered(start));
                        } else {
                            list_stack.push(ListType::Unordered);
                        }
                        // The actual list markers will be added by Tag::Item
                    }
                    Tag::Item => {
                        first_para_in_list_item=true;
                        // For all list items, we need to ensure proper formatting
                        // Determine the list type for Typst syntax
                        let list_type = match list_stack.last() {
                            Some(ListType::Ordered(start)) => match start{
                                None=> "+ ",
                                Some(1) => "+ ", // we can't differentiate 1 or default.
                                Some(num)=> &format!("{}. ",num),
                            },
                            _ => "- ",
                        };
                        
                        // Add indent based on list nesting level
                        let indent = "  ".repeat(list_stack.len() - 1);
                        
                        // Ensure the item starts on a new line
                        if !typst_output.ends_with('\n') {
                            typst_output.push('\n');
                        }
                        
                        typst_output.push_str(&format!("{}{} ", indent, list_type));
                    }
                    Tag::Emphasis => {
                        typst_output.push_str(" _");
                    }
                    Tag::Strong => {
                        in_strong_context = true;
                        handled_bold_url = false;
                        typst_output.push_str(" *"); // Keep this for non-URL content
                    }
                    Tag::Strikethrough => {
                        typst_output.push_str("#strike[");
                    }
                    Tag::Link { link_type: _, dest_url, .. } => {
                        typst_output.push_str(&format!("#link(\"{}\")[", dest_url));
                    }
                    Tag::Image { link_type, dest_url, title, .. } => {
                        // image inside a list should be indented too.
                        let list_ident = "  ".repeat(list_stack.len());
                        in_image = true;
                        log::debug!("Image link_type:{:?},dest_url:{:?},title:{:?}", link_type, dest_url, title);
                        // Extract image path
                        let image_path_name = dest_url.to_string();
                        
                        
                        // Handle local images or remote images
                        if !image_path_name.starts_with("http://") && !image_path_name.starts_with("https://")  {
                            let image_path = Path::new(&image_path_name);
                            if image_path.is_absolute() {
                                // Absolute path - use as is
                                typst_output.push_str(&format!("{},#figure(\n  image(\"{}\"),\n  caption: none)",list_ident, image_path_name));
                                
                            }else{
                                // Relative path - copy to image folder
                                let src_dir = ctx.root.join(&ctx.config.book.src);
                                let src_image_path = src_dir.join(&image_path_name);
                                
                                // Get the filename from the image path
                                // let file_name = Path::new(&image_path_name).file_name().unwrap_or_default().to_str().unwrap_or_default();
                                
                                // Destination path in the image folder
                                // let dst_image_path = image_dir.join(file_name);
                                let dst_image_path = image_dir.join(&image_path_name);

                                // get the path of the dst_image_path, create the directory if it doesn't exist
                                if let Some(image_new_path) = dst_image_path.parent(){
                                    if !image_new_path.exists() {
                                        log::debug!("Creating image dir: {:?}", image_new_path);
                                        std::fs::create_dir_all(image_new_path)?;
                                    }
                                }
                                
                                // Copy the image file
                                if src_image_path.exists() {
                                    if let Err(e) = std::fs::copy(&src_image_path, &dst_image_path) {
                                        log::error!("Failed to copy image from {:?} to {:?}: {}", src_image_path, dst_image_path, e);
                                    } else {
                                        log::debug!("Copied image from {:?} to {:?}", src_image_path, dst_image_path);
                                    }
                                } else {
                                    log::warn!("Source image not found: {:?}", src_image_path);
                                }
                                
                                // Calculate image size
                                let (width, _height) = calculate_image_size(&image_path_name,&self.max_width,&self.max_height, ctx);
                                let new_image_path = format!("{}/{}", IMAGE_DIR, image_path_name);
                                
                                typst_output.push_str(&format!("{}#figure(\n  image(\"{}\", width: {}),\n  caption: none)", 
                                    list_ident,new_image_path, width
                                ));
                                

                            }
                            
                        } else if image_path_name.starts_with("http://") || image_path_name.starts_with("https://") {
                            // Handle remote images - download to image folder
                            match download_remote_image(&image_path_name, &image_dir) {
                                Ok(local_path) => {
                                    typst_output.push_str(&format!("{}#figure(\n  image(\"{}/{}\"),\n  caption: none)", 
                                        list_ident,image_folder_name, local_path
                                    ));
                                },
                                Err(e) => {
                                    log::error!("Failed to process remote image {}: {}", image_path_name, e);
                                    typst_output.push_str(&format!("{}#text(fill: red)[Image download failed]",list_ident));
                                    
                                }
                            }
                        } 
                    }
                    Tag::Table(alignments) => {
                        // if there is a list inside a table, or a table inside a list, or nested, the situation is not handled yet.
                        log::debug!("Table alignments: {:?}", alignments);
                        table_state = TableState::InTable;
                        
                        
                        table_columns = alignments.len();
                        typst_output.push_str("#table(\n");
                        typst_output.push_str(&format!("  columns: {},\n", table_columns));
                    }
                    Tag::TableHead => {
                        log::debug!("Table columns: {}", table_columns);
                        table_state = TableState::InHeader;
                        
                        typst_output.push_str("  table.header(");
                    }
                    Tag::TableRow => {
                        table_state = TableState::InRow;
                        
                    }
                    Tag::TableCell => {
                        typst_output.push('[');
                    }
                    Tag::FootnoteDefinition(_footnote_id) => {
                        // Handle footnote definitions
                        typst_output.push_str("#footnote[");
                    }
                    _ => {}
                },
                Event::End(end_tag) => match end_tag {
                    TagEnd::Paragraph => {
                        if table_state == TableState::None {
                            typst_output.push('\n');
                        }
                    }
                    TagEnd::Heading(_) => {
                        typst_output.push('\n');
                    }
                    TagEnd::BlockQuote(_) => {
                        // Simply add the closing bracket for blockquotes
                        typst_output.push(']');
                        if !typst_output.ends_with("]\n") {
                            typst_output.push('\n');
                        }
                    }
                    TagEnd::CodeBlock => {
                        in_code_block = false;
                        is_fenced_code_block = false;
                        code_block_language = None;
                    }
                    TagEnd::List(_) => {
                        list_stack.pop();
                        typst_output.push('\n');
                    }
                    TagEnd::Item => {
                        // 在列表项结束时添加换行
                        typst_output.push('\n');
                    }
                    TagEnd::Emphasis => {
                        typst_output.push_str("_ ");
                    }
                    TagEnd::Strong => {
                        in_strong_context = false;
                        if !handled_bold_url {
                            typst_output.push_str("* "); // Only add closing asterisk if we didn't handle a URL
                        } else {
                            typst_output.push(' '); // Just add a space if we already closed the bold formatting
                        }
                    }
                    TagEnd::Strikethrough => {
                        typst_output.push(']');
                    }
                    TagEnd::Link => {
                        typst_output.push(']');
                    }
                    TagEnd::Image => {
                        // do nothing
                        in_image = false;
                    }
                    TagEnd::Table => {
                        table_state = TableState::None;
                        
                        typst_output.push_str(")\n");
                    },
                    TagEnd::TableHead => {
                        table_state = TableState::InTable;
                        typst_output.push_str("),\n");
                        
                        // // Instead of removing the trailing comma, ensure it's there but without the space
                        // if typst_output.ends_with(", ") {
                        //     // Keep the comma but replace the space with a newline
                        //     typst_output.truncate(typst_output.len() - 1);
                        //     typst_output.push('\n');
                        // } else if !typst_output.ends_with(",\n") {
                        //     // If there's no comma at all, add one before the newline
                        //     typst_output.push_str(",\n");
                        // }
                    },
                    TagEnd::TableRow => {
                        table_state = TableState::InTable;
                        
                        
                        // Instead of removing the trailing comma, ensure it's there but without the space
                        if typst_output.ends_with(", ") {
                            // Replace the space with a newline
                            typst_output.truncate(typst_output.len() - 1);
                            typst_output.push('\n');
                        } else if !typst_output.ends_with(",\n") {
                            // If there's no comma at all, add one before the newline
                            typst_output.push_str(",\n");
                        }
                    },
                    TagEnd::TableCell => {
                        typst_output.push(']');
                        typst_output.push_str(", ");
                    },
                    TagEnd::FootnoteDefinition => {
                        typst_output.push(']');
                    }
                    _ => {}
                },
                Event::Text(text) => {
                    if in_image {
                        // do nothing
                    }else if in_code_block {
                        log::debug!("chapter: {:?}, Text: {:?}", chapter.name, text);
                        if is_fenced_code_block {
                            // if it's inside a list item, ident is required.
                            let item_ident = "  ".repeat(list_stack.len());
                            typst_output.push_str(&item_ident);
                            if text.contains("```") {
                                typst_output.push_str("````");
                            }else{
                                typst_output.push_str("```");
                            }
                        }else{
                            typst_output.push_str("` ");
                        }
                        if let Some(language) = code_block_language.take() {
                            typst_output.push_str(&language);
                        }
                        if is_fenced_code_block {
                            typst_output.push('\n');
                        }
                        // typst_output.push('\n');
                        typst_output.push_str(&text);
                        if is_fenced_code_block {
                            if text.contains("```") {
                                typst_output.push_str("\n````");
                            }else{
                                typst_output.push_str("\n```");
                            }
                        }else{
                            typst_output.push_str(" `");
                        }
                    }else{
                        let text_str = text.to_string();
                    
                        if is_url(&text_str) && in_strong_context {
                            // For URLs in bold context, remove previous bold marker and format properly
                            // Check if we need to trim the previous " *" that was added
                            if typst_output.ends_with(" *") {
                                typst_output.truncate(typst_output.len() - 2);
                            }
                            
                            // Add formatted bold link
                            typst_output.push_str(&format!(" *#link(\"{}\")[{}]*", text_str, text_str));
                            handled_bold_url = true; // Mark that we've handled this bold URL
                        } else if is_url(&text_str) {
                            // Regular URL (not in bold)
                            typst_output.push_str(&format!("#link(\"{}\")[{}]", text_str, text_str));
                        } else {
                            // Regular text
                            let escaped_text = escape_typst_special_chars(&text_str);
                            let processed_text = fix_typst_formatting(&escaped_text);
                            typst_output.push_str(&processed_text);
                        }
                    }
                    
                },
                Event::Code(code) => {
                    // if it's defined as code block, no matter fenced or not, it will be handled in Event::Text following 
                    // the code block is opened and closed in Event::Text
                    typst_output.push_str("` ");
                    typst_output.push_str(&code);
                    typst_output.push_str(" `");
                },
                Event::InlineMath(math) => {
                    // Handle inline math with Typst's $ syntax
                    typst_output.push_str(&format!("${{{}}};$", math));
                },
                Event::DisplayMath(math) => {
                    // Handle display math with Typst's $$ syntax
                    typst_output.push_str(&format!("$${{{}}};$$", math));
                },
                Event::InlineHtml(html) => {
                    // Handle inline HTML similar to regular HTML
                    let html_str = html.to_string();
                    log::debug!("Inline HTML: {:?}", html_str);
                    // Check if it's an image tag
                    if html_str.contains("<img") {
                        if let Some(cap) = RE_HTML_IMG.captures(&html_str) {
                            if let Some(src) = cap.get(1) {
                                let image_path = src.as_str();
                                log::debug!("Inline image path: {:?}", image_path);
                                
                                if !image_path.starts_with("http://") && !image_path.starts_with("https://") {
                                    // Relative path
                                    let file_name = Path::new(image_path).file_name().unwrap_or_default().to_str().unwrap_or_default();
                                    
                                    // Calculate image size
                                    let (width, _height) = calculate_image_size(image_path,&self.max_width,&self.max_height, ctx);
                                    
                                    typst_output.push_str(&format!("#figure(\n  image(\"{}/{}\", width: {}),\n  caption: []\n)", 
                                        image_folder_name, file_name, width
                                    ));
                                } else {
                                    // Process remote image
                                    match download_remote_image(image_path, &image_dir) {
                                        Ok(local_path) => {
                                            typst_output.push_str(&format!("#figure(\n  image(\"{}/{}\"),\n  caption: []\n)", 
                                                image_folder_name, local_path
                                            ));
                                        },
                                        Err(e) => {
                                            log::error!("Failed to process remote image {}: {}", image_path, e);
                                            typst_output.push_str("#text(fill: red)[Image download failed]");
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Ignore other HTML tags
                },
                Event::Html(html) => {
                    // Handle block HTML
                    let html_str = html.to_string();
                    
                    // Check if it's an image tag
                    if html_str.contains("<img") {
                        if let Some(cap) = RE_HTML_IMG.captures(&html_str) {
                            if let Some(src) = cap.get(1) {
                                let image_path = src.as_str();
                                
                                if !image_path.starts_with("http://") && !image_path.starts_with("https://") {
                                    // Relative path
                                    let file_name = Path::new(image_path).file_name().unwrap_or_default().to_str().unwrap_or_default();
                                    
                                    // Calculate image size
                                    let (width, _height) = calculate_image_size(image_path, &self.max_width,&self.max_height,ctx);
                                    
                                    typst_output.push_str(&format!("#figure(\n  image(\"{}/{}\", width: {}),\n  caption: []\n)", 
                                        image_folder_name, file_name, width
                                    ));
                                } else {
                                    // Process remote image
                                    match download_remote_image(image_path, &image_dir) {
                                        Ok(local_path) => {
                                            typst_output.push_str(&format!("#figure(\n  image(\"{}/{}\"),\n  caption: []\n)", 
                                                image_folder_name, local_path
                                            ));
                                        },
                                        Err(e) => {
                                            log::error!("Failed to process remote image {}: {}", image_path, e);
                                            typst_output.push_str("#text(fill: red)[Image download failed]");
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Ignore other HTML tags
                },
                Event::FootnoteReference(reference) => {
                    typst_output.push_str(&format!("#footnote[See note {}]", reference));
                },
                Event::SoftBreak => {
                    typst_output.push(' ');
                },
                Event::HardBreak => {
                    typst_output.push_str("\\\n");
                },
                Event::Rule => {
                    typst_output.push_str("\n#line(length: 100%)\n");
                },
                Event::TaskListMarker(checked) => {
                    let marker = if checked { "[x]" } else { "[ ]" };
                    typst_output.push_str(&format!("{} ", marker));
                },
            }
        }
        
        // Apply post-processing for special typst formatting issues
        // let typst_output = post_process_typst_output(&typst_output)?;
        
        log::debug!("Converted content to typst format");
        Ok(typst_output)
    }
    // Helper function to debug book structure
    fn debug_book_structure(&self, book: &mdbook::book::Book) {
        log::debug!("Book structure analysis:");
        log::debug!("Total sections: {}", book.sections.len());
        
        for (i, item) in book.sections.iter().enumerate() {
            match item {
                mdbook::book::BookItem::Chapter(chapter) => {
                    log::debug!("Section {}: Chapter name:'{}'", i + 1, chapter.name);
                    log::debug!("  - length: {},numbers:{:?};sub_items:{}", chapter.content.len(), chapter.number, chapter.sub_items.len());
                    log::debug!("  - Path: {:?}; source_path: {:?}", chapter.path, chapter.source_path);
                    log::debug!("  - parent_names: {:?}", chapter.parent_names);
                    
                    debug_sub_items(&chapter.sub_items, 1);
                },
                mdbook::book::BookItem::Separator => {
                    log::debug!("Section {}: Separator", i + 1);
                },
                mdbook::book::BookItem::PartTitle(title) => {
                    log::debug!("Section {}: Part Title '{}'", i + 1, title);
                }
            }
        }
    }
    
    
    pub fn convert_chapters(&self,
        chapter_file_list: &mut Vec<PathBuf>, // full path of the generated typst file
        ctx: &mdbook::renderer::RenderContext
    ) -> anyhow::Result<()> {
        let book = &ctx.book;
        
        // Debug book structure
        self.debug_book_structure(book);
        
        // // Create a map to track which chapter each image belongs to
        // let mut chapter_images = std::collections::HashMap::new();
        
        for (chapter_number, item) in book.sections.iter().enumerate() {
            match item {
                mdbook::BookItem::Chapter(chapter) => {
                    
                    self.process_each_chapter(chapter, chapter_number+1, chapter_file_list, ctx)?;
                },
                mdbook::BookItem::Separator => {
                    log::debug!("Skipping separator in book structure");
                },
                mdbook::BookItem::PartTitle(title) => {
                    log::info!("Skipping part title in book structure: {}", title);
                },
            }
            
        }
        
        // // Copy images from the source directory to chapter-specific directories
        // self.copy_images(ctx, &chapter_dir, &chapter_images)?;
        
        Ok(())
    }
    // Helper function to calculate the correct relative path to templates
    fn calculate_relative_path_to_templates(&self,chapter: &mdbook::book::Chapter, ctx: &mdbook::renderer::RenderContext) -> String {
        let rel_name= self.get_chapter_relative_chapter_file_name(chapter, ctx);
        match rel_name {
            None => "".to_string(),
            Some(rel) => {
                // this folder is started from chapters, so, simply count the number of "/" or "\\" in the path
                let path_str = rel.to_string_lossy().to_string();
                let subdirs = if path_str.contains('/'){
                    path_str.split("/").count() - 1
                } else {
                    path_str.split("\\").count() - 1
                };
                "../".repeat(subdirs)
            }
        }
        
    }

    fn process_each_chapter(&self,
        chapter: &mdbook::book::Chapter,
        _chapter_number: usize, // full index, including Seperator and PartTitle
        chapter_file_list: &mut Vec<PathBuf>,
        ctx: &mdbook::renderer::RenderContext,
    ) -> anyhow::Result<()> {
        
        let chapter_dir = self.get_chapters_dir(ctx);
        if let Some(_source_path) = &chapter.source_path {
            
            let chapter_file = self.get_chapter_full_file_name(chapter, ctx).unwrap(); // when source_path is Some, the chapter_file is the full path
            chapter_file_list.push(chapter_file.clone());
            // get the path of the `typ_name` file and create the folder if not exists
            let typ_dir = chapter_file.parent().unwrap_or_else(|| &chapter_dir);
            if !typ_dir.exists() {
                std::fs::create_dir_all(typ_dir)?;
            }
            
            let typst_content = self.parse_chapter_content(chapter,&chapter.content, &chapter_file, typ_dir,ctx)?;
                
            // Write to file
            std::fs::write(&chapter_file, typst_content)?;
            log::debug!("Created typst file for {}: {}", chapter.name, chapter_file.display());

        }

        for (sub_chapter_number, sub_item) in chapter.sub_items.iter().enumerate() {
            match sub_item {
                mdbook::BookItem::Chapter(chapter) => {
                    self.process_each_chapter(chapter, sub_chapter_number+1, chapter_file_list, ctx)?;
                },
                mdbook::BookItem::Separator =>  {
                    log::debug!("Skipping separator in book structure");
                },
                mdbook::BookItem::PartTitle(title) => {
                    log::info!("Skipping part title in book structure: {}", title);
                },
            }
        }

        Ok(())
    }

    
}

fn preprocess_img_tag(content: &str) -> String {
    // Regex to capture the src attribute from img tags
    let re = regex::Regex::new(r#"<img[^>]*src=["']([^"']+)["'][^>]*>"#).unwrap();
    
    // If no img tags found, return original content
    if !re.is_match(content) {
        return content.to_string();
    }

    // Regex to extract alt attribute if it exists
    let alt_re = regex::Regex::new(r#"alt=["']([^"']+)["']"#).unwrap();

    // Replace each img tag with markdown image syntax
    let result = re.replace_all(content, |caps: &regex::Captures| {
        let src = caps.get(1).unwrap().as_str();
        let img_tag = caps.get(0).unwrap().as_str();
        
        // Try to extract alt attribute
        let alt = if let Some(alt_caps) = alt_re.captures(img_tag) {
            alt_caps.get(1).unwrap().as_str().to_string()
        } else {
            // If alt is missing, extract filename from src path
            let path = Path::new(src);
            let filename = path.file_stem().unwrap_or_default().to_str().unwrap_or_default();
            filename.to_string()
        };
        
        log::debug!("Converting img tag - src: {:?}, alt: {:?}", src, alt);
        format!("![{}]({})", alt, src)
    });

    result.into_owned()
}
// Helper function to escape special Typst characters
fn escape_typst_special_chars(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    
    // Skip leading # characters that might be from Markdown headings
    let text_without_leading_hash = text.trim_start_matches('#').trim_start();
    let text_to_process = if text_without_leading_hash.len() < text.len() {
        text_without_leading_hash
    } else {
        text
    };
    
    // Special case for "unquoted *" pattern in command line examples
    if text_to_process.contains("unquoted *") {
        return text_to_process.replace("unquoted *", "unquoted \\*");
    }
    
    // Special handling for URLs
    if is_url(text_to_process) {
        // For URLs, don't escape any characters as they'll be handled by #link
        return text_to_process.to_string();
    }
    
    // Original character escaping logic
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
    result
}

// Helper function to fix common formatting issues in Typst output
fn fix_typst_formatting(text: &str) -> String {
    let mut result = text.to_string();
    
    // Fix spacing after colons in bold text
    result = result.replace("*:", "* :");
    
    // // Fix spacing after "is" in bold text
    // result = result.replace("*is", "* is");
    
    // // Fix spacing after "pod" in bold text
    // result = result.replace("*pod", "* pod");
    
    result
}

// // Helper function to post-process the Typst output
// pub fn post_process_typst_output(content: &str) -> Result<String, Error> {
//     let mut processed_content = String::new();
//     // Remove unused state variables
//     let mut code_block_depth = 0;
//     let mut raw_call_depth = 0;
//     let mut quote_block_depth = 0;
//     let mut xml_tag_depth = 0;
    
//     // First pass: identify unclosed elements and standalone closing brackets
//     let mut standalone_closing_brackets = Vec::new();
//     let mut quote_stack = Vec::new();
    
//     for (i, line) in content.lines().enumerate() {
//         // Track state for each line
//         if line.contains("#code_block[") {
//             code_block_depth += 1;
//         }
//         if line.contains("#raw(") {
//             raw_call_depth += 1;
//         }
//         if line.contains("#quote[") {
//             quote_block_depth += 1;
//             quote_stack.push(i + 1); // Store line number (1-indexed)
//         }
        
//         // Check for closing brackets
//         let closing_brackets = line.matches("]").count();
//         if closing_brackets > 0 {
//             // Handle escaped closing brackets (like \])
//             let escaped_closing_brackets = line.matches("\\]").count();
//             let actual_closing_brackets = closing_brackets - escaped_closing_brackets;
            
//             // Adjust depths based on actual closing brackets
//             if code_block_depth > 0 && actual_closing_brackets > 0 {
//                 code_block_depth -= std::cmp::min(code_block_depth, actual_closing_brackets);
//             } else if raw_call_depth > 0 && actual_closing_brackets > 0 {
//                 raw_call_depth -= std::cmp::min(raw_call_depth, actual_closing_brackets);
//             } else if quote_block_depth > 0 && actual_closing_brackets > 0 {
//                 let brackets_to_close = std::cmp::min(quote_block_depth, actual_closing_brackets);
//                 quote_block_depth -= brackets_to_close;
//                 // Remove the corresponding opening brackets from the stack
//                 for _ in 0..brackets_to_close {
//                     if !quote_stack.is_empty() {
//                         quote_stack.pop();
//                     }
//                 }
//             } else if actual_closing_brackets > 0 {
//                 // This is a standalone closing bracket
//                 if !line.trim().starts_with("\\]") { // Ignore if it's an escaped bracket at the start of a line
//                     standalone_closing_brackets.push(i + 1); // Store line number (1-indexed)
//                 }
//             }
//         }
        
//         // Check for XML tags
//         if line.contains("<") && line.contains(">") {
//             let opening_tags = line.matches("<").count();
//             let closing_tags = line.matches(">").count();
            
//             match opening_tags.cmp(&closing_tags) {
//                 std::cmp::Ordering::Greater => {
//                     xml_tag_depth += opening_tags - closing_tags;
//                 },
//                 std::cmp::Ordering::Less => {
//                     let excess_closing = closing_tags - opening_tags;
//                     if xml_tag_depth >= excess_closing {
//                         xml_tag_depth -= excess_closing;
//                     } else {
//                         // More closing tags than we have open
//                         xml_tag_depth = 0;
//                     }
//                 },
//                 std::cmp::Ordering::Equal => {
//                     // Tags are balanced, no change to depth
//                 }
//             }
//         }
//     }
    
//     // Store unclosed elements counts for later use
//     let unclosed_code_blocks = code_block_depth;
//     let unclosed_raw_calls = raw_call_depth;
//     let unclosed_quote_blocks = quote_block_depth;
//     let unclosed_xml_tags = xml_tag_depth;
    
//     // Log warnings for standalone closing brackets
//     if !standalone_closing_brackets.is_empty() {
//         warn!("Standalone closing brackets at lines: {:?}", standalone_closing_brackets);
//         log::info!("{}",&content[0..50]);
//     }
    
//     // Reset state for second pass
//     code_block_depth = 0;
//     raw_call_depth = 0;
//     quote_block_depth = 0;
    
//     // Second pass: process content
//     for line in content.lines() {
//         let mut processed_line = line.to_string();
        
//         // Handle escaped closing brackets by temporarily replacing them
//         processed_line = processed_line.replace("\\]", "ESCAPED_BRACKET_PLACEHOLDER");
        
//         // Track state for each line
//         if processed_line.contains("#code_block[") {
//             code_block_depth += 1;
//         }
//         if processed_line.contains("#raw(") {
//             raw_call_depth += 1;
//         }
//         if processed_line.contains("#quote[") {
//             quote_block_depth += 1;
//         }
        
//         // Check for closing brackets
//         let closing_brackets = processed_line.matches("]").count();
//         if closing_brackets > 0 {
//             // Adjust depths based on closing brackets
//             if code_block_depth > 0 && closing_brackets > 0 {
//                 let brackets_to_close = std::cmp::min(code_block_depth, closing_brackets);
//                 code_block_depth -= brackets_to_close;
//             } else if raw_call_depth > 0 && closing_brackets > 0 {
//                 let brackets_to_close = std::cmp::min(raw_call_depth, closing_brackets);
//                 raw_call_depth -= brackets_to_close;
//             } else if quote_block_depth > 0 && closing_brackets > 0 {
//                 let brackets_to_close = std::cmp::min(quote_block_depth, closing_brackets);
//                 quote_block_depth -= brackets_to_close;
//             }
//         }
        
//         // Restore escaped closing brackets
//         processed_line = processed_line.replace("ESCAPED_BRACKET_PLACEHOLDER", "\\]");
        
//         // Add the processed line to the output
//         processed_content.push_str(&processed_line);
//         processed_content.push('\n');
//     }
    
//     // Close any unclosed elements at the end of the file
//     if unclosed_quote_blocks > 0 {
//         warn!("Adding {} closing brackets for unclosed quote blocks", unclosed_quote_blocks);
//         for _ in 0..unclosed_quote_blocks {
//             processed_content.push_str("]\n");
//         }
//     }
    
//     if unclosed_code_blocks > 0 {
//         warn!("Adding {} closing brackets for unclosed code blocks", unclosed_code_blocks);
//         for _ in 0..unclosed_code_blocks {
//             processed_content.push_str("]\n");
//         }
//     }
    
//     if unclosed_raw_calls > 0 {
//         warn!("Adding {} closing parentheses for unclosed raw calls", unclosed_raw_calls);
//         for _ in 0..unclosed_raw_calls {
//             processed_content.push_str(")\n");
//         }
//     }
    
//     if unclosed_xml_tags > 0 {
//         warn!("Adding {} closing tags for unclosed XML tags", unclosed_xml_tags);
//         for _ in 0..unclosed_xml_tags {
//             processed_content.push_str(">\n");
//         }
//     }
    
//     Ok(processed_content)
// }

// // New function to convert MathJax notation to Typst math syntax
// fn convert_mathjax_to_typst(text: &str) -> String {
//     let mut result = text.to_string();
    
//     // Convert display math: \\[ ... \\] to Typst's $ ... $ format
//     // Use (?s) flag for multiline matching and .*? for non-greedy matching
//     let display_math_re = regex::Regex::new(r"(?s)\\\\(\[)(.*?)\\\\(\])").unwrap();
//     result = display_math_re.replace_all(&result, |caps: &regex::Captures| {
//         let math_content = &caps[2];
//         format!("$ {} $", convert_latex_to_typst(math_content))
//     }).to_string();
    
//     // Convert inline math: \\( ... \\) to Typst's $ ... $ format
//     // Use (?s) flag for multiline matching and .*? for non-greedy matching
//     let inline_math_re = regex::Regex::new(r"(?s)\\\\(\()(.*?)\\\\(\))").unwrap();
//     result = inline_math_re.replace_all(&result, |caps: &regex::Captures| {
//         let math_content = &caps[2];
//         format!("$ {} $", convert_latex_to_typst(math_content))
//     }).to_string();
    
//     result
// }

// // Helper function to convert LaTeX math syntax to Typst math syntax
// fn convert_latex_to_typst(latex: &str) -> String {
//     // This is a simplified converter that handles common LaTeX constructs
//     // A complete converter would be more extensive
//     let mut typst = latex.to_string();
    
//     // Convert common LaTeX commands to Typst equivalents
//     // Basic replacements
//     typst = typst.replace("\\begin{aligned}", "");
//     typst = typst.replace("\\end{aligned}", "");
    
//     // Matrices
//     typst = typst.replace("\\begin{pmatrix}", "mat(");
//     typst = typst.replace("\\end{pmatrix}", ")");
    
//     // Better matrix handling with proper comma separation
//     // Replace matrix row separators \\ with ), (
//     let matrix_row_re = regex::Regex::new(r"mat\((.*?)\\\\").unwrap();
//     typst = matrix_row_re.replace_all(&typst, |caps: &regex::Captures| {
//         format!("mat({});", &caps[1])
//     }).to_string();
    
//     // Convert the cells within a matrix row to be comma-separated
//     let matrix_cell_re = regex::Regex::new(r"mat\((.*?)\s+(\S+)\s*\)").unwrap();
//     typst = matrix_cell_re.replace_all(&typst, |caps: &regex::Captures| {
//         let cells = caps[1].split_whitespace().collect::<Vec<&str>>().join(", ");
//         let last_cell = &caps[2];
//         format!("mat({}, {})", cells, last_cell)
//     }).to_string();
    
//     // Fractions
//     let frac_re = regex::Regex::new(r"\\frac\{([^}]*)\}\{([^}]*)\}").unwrap();
//     typst = frac_re.replace_all(&typst, |caps: &regex::Captures| {
//         format!("frac({}, {})", &caps[1], &caps[2])
//     }).to_string();
    
//     // Subscripts and superscripts
//     let subscript_re = regex::Regex::new(r"_\{([^}]*)\}").unwrap();
//     typst = subscript_re.replace_all(&typst, |caps: &regex::Captures| {
//         format!("_({}) ", &caps[1])
//     }).to_string();
    
//     let superscript_re = regex::Regex::new(r"\^\{([^}]*)\}").unwrap();
//     typst = superscript_re.replace_all(&typst, |caps: &regex::Captures| {
//         format!("^({}) ", &caps[1])
//     }).to_string();
    
//     // Common mathematical symbols
//     typst = typst.replace("\\infty", "infinity");
//     typst = typst.replace("\\int", "integral");
//     typst = typst.replace("\\sum", "sum");
//     typst = typst.replace("\\prod", "product");
//     typst = typst.replace("\\to", "arrow");
//     typst = typst.replace("\\rightarrow", "arrow");
//     typst = typst.replace("\\Rightarrow", "=>"); 
//     typst = typst.replace("\\implies", "=>");
//     typst = typst.replace("\\alpha", "alpha");
//     typst = typst.replace("\\beta", "beta");
//     typst = typst.replace("\\gamma", "gamma");
//     typst = typst.replace("\\delta", "delta");
//     typst = typst.replace("\\epsilon", "epsilon");
//     typst = typst.replace("\\zeta", "zeta");
//     typst = typst.replace("\\eta", "eta");
//     typst = typst.replace("\\theta", "theta");
//     typst = typst.replace("\\iota", "iota");
//     typst = typst.replace("\\kappa", "kappa");
//     typst = typst.replace("\\lambda", "lambda");
//     typst = typst.replace("\\mu", "mu");
//     typst = typst.replace("\\nu", "nu");
//     typst = typst.replace("\\xi", "xi");
//     typst = typst.replace("\\pi", "pi");
//     typst = typst.replace("\\rho", "rho");
//     typst = typst.replace("\\sigma", "sigma");
//     typst = typst.replace("\\tau", "tau");
//     typst = typst.replace("\\upsilon", "upsilon");
//     typst = typst.replace("\\phi", "phi");
//     typst = typst.replace("\\chi", "chi");
//     typst = typst.replace("\\psi", "psi");
//     typst = typst.replace("\\omega", "omega");
    
//     // Align markers
//     typst = typst.replace("&=", "=");
//     typst = typst.replace("&\\approx", "approx");
    
//     // Line breaks in math
//     typst = typst.replace("\\\\", " \\\n");
    
//     typst
// }

// // 辅助函数，只处理非代码块中的$符号
// fn escape_dollar_signs(text: &str) -> String {
//     let re = regex::Regex::new(r"\$([A-Za-z][A-Za-z0-9_]*)").unwrap();
    
//     let mut result = String::new();
//     let mut last_end = 0;
    
//     for cap in re.captures_iter(text) {
//         let full_match = cap.get(0).unwrap();
//         let start = full_match.start();
//         let end = full_match.end();
        
//         // 检查$是否已经被转义
//         let is_escaped = start > 0 && text[..start].ends_with('\\');
        
//         // 添加匹配前的文本
//         result.push_str(&text[last_end..start]);
        
//         // 添加匹配，根据需要转义$
//         if is_escaped {
//             // 已经转义，保持原样
//             result.push_str(&text[start..end]);
//         } else {
//             // 未转义，添加转义
//             result.push('\\');
//             result.push_str(&text[start..end]);
//         }
        
//         last_end = end;
//     }
    
//     // 添加剩余文本
//     result.push_str(&text[last_end..]);
    
//     result
// }

// Helper function to recursively debug sub-items
fn debug_sub_items(sub_items: &[mdbook::book::BookItem], level: usize) {
    let indent = "  ".repeat(level + 1);
    
    for (i, item) in sub_items.iter().enumerate() {
        match item {
            mdbook::book::BookItem::Chapter(chapter) => {
                log::debug!("{}Sub-item {}: Chapter '{}'", indent, i + 1, chapter.name);
                log::debug!("  - length: {},numbers:{:?};sub_items:{}", chapter.content.len(), chapter.number, chapter.sub_items.len());
                log::debug!("  - Path: {:?}; source_path: {:?}", chapter.path, chapter.source_path);
                log::debug!("  - parent_names: {:?}", chapter.parent_names);
                
                if !chapter.sub_items.is_empty() {
                    debug_sub_items(&chapter.sub_items, level + 1);
                }
            },
            mdbook::book::BookItem::Separator => {
                log::debug!("{}Sub-item {}: Separator", indent, i + 1);
            },
            mdbook::book::BookItem::PartTitle(title) => {
                log::debug!("{}Sub-item {}: Part Title '{}'", indent, i + 1, title);
            }
        }
    }
}

// Function to download an image from URL and save it locally
fn download_remote_image(image_url: &str, image_dir: &Path) -> anyhow::Result<String> {
    // Create a client for making requests
    let client = Client::new();
    
    // Generate a unique filename based on URL hash or timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // Extract extension from URL or default to png
    let extension = match image_url.split('.').last() {
        Some(ext) if ["jpg", "jpeg", "png", "gif", "webp", "svg"].contains(&ext.to_lowercase().as_str()) => ext,
        _ => "png",
    };
    
    let file_name = format!("remote_img_{}.{}", timestamp, extension);
    let file_path = image_dir.join(&file_name);
    
    // Attempt to download the image
    match client.get(image_url).send() {
        Ok(response) => {
            if response.status().is_success() {
                match response.bytes() {
                    Ok(bytes) => {
                        // Write the image data to file
                        let mut file = fs::File::create(&file_path)?;
                        file.write_all(bytes.as_ref())?;
                        
                        // Return the filename to be used in Typst
                        log::debug!("Downloaded remote image: {} -> {}", image_url, file_path.display());
                        Ok(file_name)
                    },
                    Err(e) => {
                        log::error!("Failed to read image bytes from {}: {}", image_url, e);
                        create_placeholder_image(image_dir)
                    }
                }
            } else {
                log::error!("Failed to download image {}: HTTP status {}", image_url, response.status());
                create_placeholder_image(image_dir)
            }
        },
        Err(e) => {
            log::error!("Failed to download image {}: {}", image_url, e);
            create_placeholder_image(image_dir)
        }
    }
}

// Create a placeholder image when download fails
fn create_placeholder_image(image_dir: &Path) -> anyhow::Result<String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let file_name = format!("placeholder_{}.txt", timestamp);
    let file_path = image_dir.join(&file_name);
    
    // Create a simple text file that Typst can include
    let mut file = fs::File::create(&file_path)?;
    file.write_all(b"#text(fill: red)[Image could not be downloaded]")?;
    
    log::warn!("Created placeholder for failed image download: {}", file_path.display());
    Ok(file_name)
}

// Add this function to detect URLs
fn is_url(text: &str) -> bool {
    // Simple check for http/https URLs
    text.starts_with("http://") || text.starts_with("https://")
}

// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_img_tag() {
        let input = r###"<img src="docs/01-introduction/image-20250224001420194.png" alt="image-20250224001420194" style="zoom:50%;" />"###;
        let expected = r###"![image-20250224001420194](docs/01-introduction/image-20250224001420194.png)"###;
        let output = preprocess_img_tag(input);
        assert_eq!(output, expected);

        let input = r###"## Work Folder and Common OS Environment Variables

<img src="docs/01-introduction/image-20250221105725169.png" alt="image-20250221105725169" style="zoom:50%;" />

Please download the attached small zip file and unzip it to a **working folder** you created. Under the working folder, you should see several components:"###;
        let expected = r###"## Work Folder and Common OS Environment Variables

![image-20250221105725169](docs/01-introduction/image-20250221105725169.png)

Please download the attached small zip file and unzip it to a **working folder** you created. Under the working folder, you should see several components:"###;
        let output = preprocess_img_tag(input);
        assert_eq!(output, expected);

        // <img src="_images/image-20250213001741756.png" style="zoom:50%;" />
        let input = r###"<img src="_images/image-20250213001741756.png" style="zoom:50%;" />"###;
        let expected = r###"![image-20250213001741756](_images/image-20250213001741756.png)"###;
        let output = preprocess_img_tag(input);
        assert_eq!(output, expected);
    }
}