use std::path::{Path, PathBuf};

use super::{Config, BEST_PRACTICE_TEMPLATE,  TARGET_CHAPTERS_DIR, TARGET_TEMPLATE_DIR};

impl Config {
    pub fn get_chapter_full_file_name(&self, chapter:&mdbook::book::Chapter, ctx: &mdbook::renderer::RenderContext) -> Option<PathBuf> {
        let chapter_dir = self.get_chapters_dir(ctx);
        match &chapter.source_path {
            None => None,
            Some(source_path) => {
                let chapter_file = chapter_dir.join(source_path.with_extension("typ"));
                // let chapter_file = chapter_dir.join(chapter_file_name);
                Some(chapter_file)
            }
        }
    }

    pub fn get_chapter_relative_chapter_file_name(&self, chapter:&mdbook::book::Chapter, _ctx: &mdbook::renderer::RenderContext) -> Option<PathBuf> {
        let chapter_dir = PathBuf::from(TARGET_CHAPTERS_DIR);
        match &chapter.source_path {
            None => None,
            Some(source_path) => {
                let chapter_file = chapter_dir.join(source_path.with_extension("typ")); // chapters/chapter_1.typ
                // let chapter_file = chapter_dir.join(chapter_file_name);
                Some(chapter_file)
            }
        }
    }
    pub fn append_chapter_to_typst_output(&self, ctx: &mdbook::renderer::RenderContext, typst_output: &mut String) -> anyhow::Result<()> {
        let book = &ctx.book;
        let mut typ_content = Vec::new();
        
        for item in book.sections.iter() {
            self.process_book_item(item, &mut typ_content, ctx)?;
        }
        
        // Add all include directives to the typst output
        for line in typ_content {
            typst_output.push_str(&format!("{}\n", line));
        }
        
        Ok(())
    }
    
    // Helper function to recursively process book items (chapters, sections)
    fn process_book_item(&self, item: &mdbook::book::BookItem, typ_content: &mut Vec<String>, ctx: &mdbook::renderer::RenderContext) -> anyhow::Result<()> {
        match item {
            mdbook::book::BookItem::Chapter(chapter) => {
                if let Some(chapter_path) = self.get_chapter_relative_chapter_file_name(chapter, ctx) {
                    log::debug!("Including chapter: {} with path: {}", chapter.name, chapter_path.display());
                
                    // Add include directive for the chapter with its original path under "chapter"
                    typ_content.push(format!("#include \"{}\"", chapter_path.to_string_lossy().replace('\\', "/")));
                }
                
                
                // Process sub-items recursively
                for sub_item in &chapter.sub_items {
                    self.process_book_item(sub_item, typ_content, ctx)?;
                }
            },
            mdbook::book::BookItem::Separator => {
                // Just log that we're skipping the separator
                log::info!("Skipping separator in book structure");
            },
            mdbook::book::BookItem::PartTitle(title) => {
                // Add part title as a comment
                log::info!("Adding part title: {}", title);
                typ_content.push(format!("// Part: {}", title));
            }
        }
        Ok(())
    }

    
    pub fn convert_book(&self, _chapter_file_list: &mut [PathBuf], ctx: &mdbook::renderer::RenderContext) -> anyhow::Result<()> {
        let target_template_dir = self.get_typst_templates_dir(ctx);
        if !target_template_dir.exists() {
            return Err(anyhow::anyhow!("template directory {} not found", target_template_dir.display()));
        }
        
        // // Convert all markdown files preserving their original paths
        // self.convert_all_chapters(ctx)?;
        
        // check the templates hashmap
        if self.templates.is_empty() {
            let mut typst_output = String::new();
            
            // Don't add package imports here as they're now in each chapter file
            // Just include the chapters
            self.append_chapter_to_typst_output(ctx, &mut typst_output)?;

            // write the typst_output to the file
            self.write_typst_file(ctx, &typst_output, None)?;
        } else {
            for (name, file_name) in &self.templates {
                let mut typst_output = String::new();
                // get the file name from the template_file 
                let file_name = Path::new(file_name).file_name().unwrap().to_str().unwrap();
                let dst = target_template_dir.join(file_name);
                // if the dst file doesn't exist or is not a file, error out
                if !dst.exists() || !dst.is_file() {
                    return Err(anyhow::anyhow!("template file {} not found", dst.display()));
                }

                typst_output.push_str(&format!("#import \"{}/{}\": {}\n",TARGET_TEMPLATE_DIR,file_name,BEST_PRACTICE_TEMPLATE));
                
                // Templates still need their metadata and setup
                typst_output.push_str("\n\n//Document Metadata\n");

                typst_output.push_str("#let metadata = (\n");
                for (key, value) in &self.template_parameters {
                    // please escape the value if it contains double quotes
                    let value = value.replace("\"", "\\\"");
                    // please escape the \n in the value with \\n
                    let value = value.replace("\n", "\\n");
                    typst_output.push_str(&format!("    {}: \"{}\",\n", key, value));
                }
                typst_output.push_str(")\n");

                typst_output.push_str(&format!("#show: doc => {}(\n",BEST_PRACTICE_TEMPLATE));
                for key in self.template_parameters.keys() {
                    typst_output.push_str(&format!("    {}: metadata.{},\n",key,key));
                }
                typst_output.push_str("  doc\n");
                typst_output.push_str(")\n\n");

                // append all chapter files to the typst_output
                self.append_chapter_to_typst_output(ctx, &mut typst_output)?;

                // write the typst_output to the file
                self.write_typst_file(ctx, &typst_output, Some(name))?;
            }
        }

        Ok(())
    }

    pub fn write_typst_file(&self, ctx: &mdbook::renderer::RenderContext, typst_output: &str, template_name:Option<&str>) -> anyhow::Result<()> {
        let book_name = self.get_book_name(template_name, ctx);
        let typst_dir = self.get_typst_dir(ctx);
        let output_file = typst_dir.join(format!("{}.typ", book_name));
        std::fs::write(output_file, typst_output).map_err(|e| anyhow::anyhow!("failed to write typst file:{}", e))
        // Ok(())
    }

    pub fn convert_book_to_pdf(&self, ctx: &mdbook::renderer::RenderContext) -> anyhow::Result<()> {
        if self.templates.is_empty() {
            self.invoke_typst_command(ctx,  None)?;
        }else {
            for name in self.templates.keys() {
                self.invoke_typst_command(ctx,  Some(name))?;
            }
        }
        // log::info!("destination:{}", ctx.destination.display());
        Ok(())
    }

    pub fn invoke_typst_command(&self, ctx: &mdbook::renderer::RenderContext,  template_name:Option<&str>) -> anyhow::Result<()> {
        let book_name = self.get_book_name(template_name, ctx);
        let typst_dir = self.get_typst_dir(ctx);
        let typst_file = typst_dir.join(format!("{}.typ", book_name));
        if !typst_file.exists() || !typst_file.is_file() {
            return Err(anyhow::anyhow!("typst file {} not found", typst_file.display()));
        }
        let pdf_dir = self.get_pdf_dir(ctx);
        // create the pdf_dir if it doesn't exist
        if !pdf_dir.exists() {
            std::fs::create_dir_all(&pdf_dir)?;
        }
        let output_file = pdf_dir.join(format!("{}.pdf", book_name));
        // run the typst command to convert the typst file to pdf
        let status = std::process::Command::new("typst").arg("compile").arg(&typst_file).arg(&output_file).status()?;
        if !status.success() {
            return Err(anyhow::anyhow!("failed to convert typst file:{} to pdf", typst_file.display()));
        }
        log::info!("converted typst file:{} to pdf:{}", typst_file.display(), output_file.display());
        Ok(())
    }

    
}