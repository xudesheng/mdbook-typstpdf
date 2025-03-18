
use std::fs::File;

use anyhow::{anyhow, Context as _};

use once_cell::sync::Lazy;
mod config;
pub use config::Config;


#[derive(Default)]
pub struct Renderer {
    logfile: Option<File>,
}

impl Renderer {
    pub fn new() -> Self {
        Self { logfile: None }
    }

    const NAME: &'static str = "typstpdf";
    const CONFIG_KEY: &'static str = "output.typstpdf";
}


impl mdbook::Renderer for Renderer {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn render(&self, ctx: &mdbook::renderer::RenderContext) -> anyhow::Result<()> {
        if self.logfile.is_none() {
            log::info!("no logfile");
        }
        // If we're compiled against mdbook version I.J.K, require ^I.J
        // This allows using a version of mdbook with an earlier patch version as a server
        static MDBOOK_VERSION_REQ: Lazy<semver::VersionReq> = Lazy::new(|| {
            let compiled_mdbook_version = semver::Version::parse(mdbook::MDBOOK_VERSION).unwrap();
            semver::VersionReq {
                comparators: vec![semver::Comparator {
                    op: semver::Op::Caret,
                    major: compiled_mdbook_version.major,
                    minor: Some(compiled_mdbook_version.minor),
                    patch: None,
                    pre: Default::default(),
                }],
            }
        });
        let mdbook_server_version = semver::Version::parse(&ctx.version).unwrap();
        if !MDBOOK_VERSION_REQ.matches(&mdbook_server_version) {
            log::warn!(
                "{} is semver-incompatible with mdbook {} (requires {})",
                env!("CARGO_PKG_NAME"),
                mdbook_server_version,
                *MDBOOK_VERSION_REQ,
            );
        }

        let cfg: Config = ctx
            .config
            .get_deserialized_opt(Self::CONFIG_KEY)
            .with_context(|| format!("Unable to deserialize {}", Self::CONFIG_KEY))?
            .ok_or(anyhow!("No {} table found", Self::CONFIG_KEY))?;

        log::info!("cfg: {:?}", cfg);
        
        // create a folder: `typst-pdf` under the book folder in mdbook
        
        // let typst_pdf_dir = cfg.get_output_dir(&ctx);
        // fs::create_dir_all(&typst_pdf_dir)?;
        // log::info!("typst_pdf root dir: {}", typst_pdf_dir.display());

        // Create a simplified context with only the fields we need
       

        // for item in book.sections.iter() {
        //     log::info!("item: {:?}", item);
        // }

        // Ok(())
        cfg.renderer(ctx)
    }
}
