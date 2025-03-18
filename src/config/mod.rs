use std::{
    collections::HashMap,
    path::PathBuf,
};

pub mod renderer;
pub mod chapter;
pub mod book;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// the typst template directory that holds all templates you can reference
    #[serde(rename = "template_dir",default = "get_default_template_dir")]
    pub template_dir: String, // the directory of the templates, default is "./typst-template"

    /// the list of template name and template file name. 
    /// this tool will generate a pdf file for each template
    /// if no template is provided, it will generate a default pdf file without template.
    #[serde(rename = "templates", default = "Default::default")]
    pub templates: HashMap<String, String>, // template name -> template path, the template name will be the name for pdf output

    /// whether the intermidate typst files for each chapter should be kept or not.
    #[serde(rename = "keep_typst_files",default = "Default::default")]
    pub keep_typst_files: bool, // whether to keep the preprocessed files, typst files

    /// parameter and value pairs for template use.
    /// you can define your value pair that match your template definition.
    #[serde(rename = "template_parameters",default = "get_default_template_parameters")]
    pub template_parameters: HashMap<String, String>, // the parameters for the template, default is empty

    /// option multi-lines string that can be imported at the beginning of each chapter. 
    /// it's very useful when you want to add the import statements for popular typst external functions.
    #[serde(rename = "chapter_imports")]
    pub chapter_imports: Option<String>, // it will be a multi-lines string

    /// max_width in a floating number between 0.0 and 1.0 (include). 
    /// this constraint will only be applied when the max height already meets the requirement.
    #[serde(rename = "max_width",default = "Default::default")]
    pub max_width: Option<f64>,

    /// max_height in a floating number between 0.0 and 1.0 (include).
    #[serde(rename = "max_height",default = "Default::default")]
    pub max_height: Option<f64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // output_dir: get_default_output_dir(),
            template_dir: get_default_template_dir(),
            templates: HashMap::new(),
            keep_typst_files: false,
            template_parameters: get_default_template_parameters(),
            chapter_imports: None,
            max_width: None,
            max_height: None,
        }
    }
}
pub const TARGET_TEMPLATE_DIR: &str = "templates";
pub const TARGET_CHAPTERS_DIR: &str = "chapters";
pub const TARGET_TYPST_DIR: &str = "typst";
pub const TARGET_PDF_DIR: &str = "pdf";
pub const BEST_PRACTICE_TEMPLATE:&str = "best_practice_template";

pub const IMAGE_DIR:&str = "__images";

impl Config {
    pub fn get_book_name(&self,template_name: Option<&str>, ctx: &mdbook::renderer::RenderContext) -> String {
        // doesn't include the extension
        // it can be typ or pdf
        let root_name = ctx.root.file_name().unwrap().to_str().unwrap();
        let root_name = if root_name.is_empty() {
            "book"
        } else {
            root_name
        };
        match template_name {
            Some(template_name) => format!("{}-{}",root_name,template_name),
            None => root_name.to_string(),
        }
    }
    pub fn get_template_dir(&self, ctx: &mdbook::renderer::RenderContext) -> PathBuf {
        // this is the directory of the source templates
        let template_path = std::path::Path::new(&self.template_dir);
        
        if template_path.is_absolute() {
            PathBuf::from(&self.template_dir)
        } else {
            ctx.root.clone().join(&self.template_dir)
        }
    }

    pub fn get_typst_dir(&self, ctx: &mdbook::renderer::RenderContext) -> PathBuf {
        
        let typst_pdf_dir = self.get_output_dir(ctx);
        
        typst_pdf_dir.join(TARGET_TYPST_DIR)
    }

    pub fn get_pdf_dir(&self, ctx: &mdbook::renderer::RenderContext) -> PathBuf {
        let typst_dir = self.get_output_dir(ctx);
        typst_dir.join(TARGET_PDF_DIR)
    }

    pub fn get_typst_templates_dir(&self, ctx: &mdbook::renderer::RenderContext) -> PathBuf {
        // this is the directory of templates to support final typst outputs
        let typst_dir = self.get_typst_dir(ctx);
        typst_dir.join(TARGET_TEMPLATE_DIR)
    }

    pub fn get_chapters_dir(&self, ctx: &mdbook::renderer::RenderContext) -> PathBuf {
        let typst_dir = self.get_typst_dir(ctx);
        typst_dir.join(TARGET_CHAPTERS_DIR)
    }

    pub fn get_output_dir(&self, ctx: &mdbook::renderer::RenderContext) -> PathBuf {
        ctx.destination.clone()
    }
}
// fn get_default_output_dir() -> Vec<String> {
//     vec!["book".to_string(),"pdf-output".to_string()]
// }


fn get_default_template_parameters() -> HashMap<String, String> {
    let mut result = HashMap::new();
    // doc_title: "Document Title",
    // doc_version: "1.0",
    // abstract: "Document abstract",
    // doc_author: "Author Name",
    // author_email: "author@example.com",
    // doc_date: "January 1, 2023",
    // software_tested: "Software v1.0",
    // feedback_email: "feedback@example.com",
    // reviewers: "Reviewer Names",
    result.insert("doc_title".to_string(), "Document Title".to_string());
    result.insert("doc_version".to_string(), "1.0".to_string());
    result.insert("abstract".to_string(), "Document abstract".to_string());
    result.insert("doc_author".to_string(), "Author Name".to_string());
    result.insert("author_email".to_string(), "author@example.com".to_string());
    result.insert("doc_date".to_string(), "January 1, 2023".to_string());
    result.insert("software_tested".to_string(), "Software v1.0".to_string());
    result.insert("feedback_email".to_string(), "feedback@example.com".to_string());
    result.insert("reviewers".to_string(), "Reviewer Names".to_string());
    result
}

fn get_default_template_dir() -> String {
    "./typst-template".to_string()
}
