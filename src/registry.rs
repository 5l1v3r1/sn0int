use errors::*;
use args::{Args, Publish, Install};
use api::{API_URL, Client};
use auth;
use sn0int_common::metadata::Metadata;
use std::fs;
use std::path::Path;
use paths;
use term;
use worker;


pub fn run_publish(_args: &Args, publish: &Publish) -> Result<()> {
    let session = auth::load_token()
        .context("Failed to load auth token")?;

    let mut client = Client::new(API_URL)?;
    client.authenticate(session);

    let path = Path::new(&publish.path);
    let name = path.file_stem().ok_or(format_err!("Couldn't get file name"))?;
    let ext = path.extension().ok_or(format_err!("Couldn't get file extension"))?;

    if ext != "lua" {
        bail!("File extension has to be .lua");
    }

    let name = name.to_os_string().into_string()
        .map_err(|_| format_err!("Failed to decode file name"))?;

    let code = fs::read_to_string(path)
        .context("Failed to read module")?;
    let metadata = code.parse::<Metadata>()?;

    let label = format!("Uploading {} {} ({:?})", name, metadata.version, path);
    let result = worker::spawn_fn(&label, || {
        client.publish_module(&name, code.to_string())
    }, false)?;

    term::info(&format!("Published as {}/{} {}", result.author,
                                                 result.name,
                                                 result.version));

    Ok(())
}

pub fn run_install(_args: &Args, install: &Install) -> Result<()> {
    let client = Client::new(API_URL)?;

    let label = format!("Installing {}", install.module);
    worker::spawn_fn(&label, || {
        let version = match install.version {
            Some(ref version) => version.to_string(),
            None => client.query_module(&install.module)
                        .context("Failed to query module infos")?
                        .latest
                        .ok_or(format_err!("Module doesn't have a latest version"))?,
        };

        let module = client.download_module(&install.module, &version)
            .context("Failed to download module")?;

        let path = paths::module_dir()?
            .join(format!("{}.lua", install.module));

        fs::create_dir_all(path.parent().unwrap())
            .context("Failed to create folder")?;

        fs::write(&path, module.code)
            .context(format_err!("Failed to write to {:?}", path))?;

        Ok(())
    }, false)
}
