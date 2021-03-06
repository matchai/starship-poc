use crate::modules::ModuleType;
use crate::{config, prompt, vcs};
use crate::{
    error::{self, ConfigError},
    vcs::VcsInstance,
};

use anyhow::{Context as anyhow_context, Result};
use serde::de;

use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Context {
    pub current_dir: PathBuf,
    pub vcs_instance: Option<VcsInstance>,
    pub prompt_opts: prompt::PromptOpts,
    pub prompt_config: Option<toml::Value>,
}

/// Context contains data or common functions that may be used by multiple modules.
/// The data contained within Context will be relevant to this particular rendering
/// of the prompt.
impl Context {
    /// Create a new instance of context given a set of prompt options
    pub fn new(prompt_opts: prompt::PromptOpts) -> Self {
        let current_dir = Self::get_current_dir().expect("Unable to get current directory");
        let vcs_instance = vcs::get_vcs_instance(&current_dir);

        let prompt_config = config::load_prompt_config();

        Context {
            current_dir,
            vcs_instance,
            prompt_opts,
            prompt_config,
        }
    }

    pub fn load_config<'de, T>(&self, module: &impl ModuleType) -> T
    where
        T: de::Deserialize<'de> + Default,
    {
        // Extract the map associated with the module
        let module_config = match &self.prompt_config {
            Some(config) => config.get(module.metadata().name).map(|v| v.to_owned()),
            None => None,
        };

        match module_config {
            Some(config) => config.try_into().unwrap_or_else(|e| {
                log::error!(
                    "Unable to parse config for {}: {}",
                    module.metadata().name,
                    e
                );
                error::queue(ConfigError::InvalidToml(e));
                Default::default()
            }),
            None => {
                log::debug!("No config available for {}", module.metadata().name);
                Default::default()
            }
        }
    }

    fn get_current_dir() -> Result<PathBuf> {
        // Get the logical directory from `$PWD`
        env::var("PWD")
            .map(PathBuf::from)
            // Otherwise, fallback to getting the physical directory from `env::current_dir()`
            .or_else(|_err| env::current_dir().context("Unable to resolve env::current_dir()"))
    }
}
