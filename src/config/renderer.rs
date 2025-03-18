
use std::fs;

use super::Config;

impl Config {
    pub fn prepare_chapter_dir(&self, ctx: &mdbook::renderer::RenderContext) -> anyhow::Result<()> {
        
        let chapter_dir = self.get_chapters_dir(ctx);
        // create chapter dir
        fs::create_dir_all(&chapter_dir)?;
        log::debug!("created chapter dir: {}", chapter_dir.display());
        Ok(())
    }

    pub fn prepare_template_images(&self, ctx: &mdbook::renderer::RenderContext) -> anyhow::Result<()> {
        let target_template_dir = self.get_typst_templates_dir(ctx);
        let source_template_dir = self.get_template_dir(ctx);
        let images_dir = source_template_dir.join("images");
        if images_dir.exists() {
            // copy the folder to the typst_dir
            fs::create_dir_all(target_template_dir.join("images"))?;
            let entries = fs::read_dir(&images_dir)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                let dest = target_template_dir.join("images").join(path.file_name().unwrap());
                if path.is_file() {
                    fs::copy(&path, &dest)?;
                } else if path.is_dir() {
                    fs::create_dir_all(&dest)?;
                    let subentries = fs::read_dir(&path)?;
                    for subentry in subentries {
                        let subentry = subentry?;
                        let subpath = subentry.path();
                        fs::copy(&subpath, dest.join(subpath.file_name().unwrap()))?;
                    }
                }
            }
            log::debug!("copied images folder from {} to {}", images_dir.display(), target_template_dir.join("images").display());
        }

        Ok(())
    }

    pub fn prepare_templates(&self, ctx: &mdbook::renderer::RenderContext) -> anyhow::Result<()> {
        
        let target_template_dir = self.get_typst_templates_dir(ctx);
        // create it if not exists
        fs::create_dir_all(&target_template_dir)?;
        let source_template_dir = self.get_template_dir(ctx);
        log::debug!("template_dir value:{},template_dir folder: {}", self.template_dir, source_template_dir.display());

        self.prepare_template_images(ctx)?;
        
        // check the templates hashmap
        for (name, file_name) in &self.templates {
            let template_file = source_template_dir.join(file_name);
            if !template_file.exists() {
                return Err(anyhow::anyhow!("template file {} not found", template_file.display()));
            }
            // get the file name from the template_file 
            let file_name = template_file.file_name().unwrap().to_str().unwrap();
            let dst = target_template_dir.join(file_name);
            fs::copy(&template_file, &dst)?;
            log::debug!("{},copied template file {} to {}", name, template_file.display(), dst.display());
        }

        Ok(())
    }

    pub fn renderer(&self, ctx: &mdbook::renderer::RenderContext) -> anyhow::Result<()> {
        let book = &ctx.book;
        log::debug!("book items:{}", book.sections.len());
        // steps
        // 1. create "typst" folder under the typst_pdf_dir for hosting all typst files
        // 2. copy all template typst files to the "typst" folder
        // 3. copy the images folder to support typst template to the "typst" folder
        // 4. copy code block template to the "typst" folder
        self.prepare_templates(ctx)?;
        
        // 5. create chapter folder under the typst_pdf_dir/typst/
        self.prepare_chapter_dir(ctx)?;

        // 6. convert each chapter to typst file
        let mut chapter_file_list = Vec::new();
        self.convert_chapters(&mut chapter_file_list, ctx)?;
        log::debug!("chapter_file_list: {:?}", chapter_file_list);

        // 7. convert the book to a typst file
        self.convert_book(&mut chapter_file_list,ctx)?;

        // 8. convert the book to pdf
        self.convert_book_to_pdf(ctx)?;

        if !self.keep_typst_files {
            // 9. remove the typst folder
            let typst_dir = self.get_typst_dir(ctx);
            if typst_dir.exists() {
                fs::remove_dir_all(&typst_dir)?;
                log::info!("removed typst folder: {}", typst_dir.display());
            }
        }
        
        Ok(())
    }
}
