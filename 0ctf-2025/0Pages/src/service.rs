use crate::model::SiteManifest;
use salvo::prelude::*;
use std::{
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};
use tempfile::NamedTempFile;
use zip::ZipArchive;

pub async fn get_username_from_session(depot: &mut Depot) -> Option<String> {
    if let Some(session) = depot.session_mut() {
        if let Some(username) = session.get::<String>("username") {
            if username.chars().all(char::is_alphanumeric) {
                return Some(username);
            }
            session.remove("username");
        }
    }
    None
}

pub async fn list_sites(username: &str) -> Result<Vec<SiteManifest>, std::io::Error> {
    let mut sites: Vec<SiteManifest> = Vec::new();
    let mut dir = tokio::fs::read_dir("data").await?;
    while let Ok(Some(entry)) = dir.next_entry().await {
        if entry.file_type().await?.is_dir() {
            let site_id = entry.file_name().to_str().unwrap_or_default().to_string();
            let manifest_path = format!("data/{}/manifest.json", site_id);
            if let Ok(manifest_data) = tokio::fs::read_to_string(&manifest_path).await {
                if let Ok(mut manifest) = serde_json::from_str::<SiteManifest>(&manifest_data) {
                    if manifest.owner == Some(username.to_string()) {
                        manifest.site_id = Some(site_id.clone());
                        sites.push(manifest);
                    }
                }
            }
        }
    }
    Ok(sites)
}

pub async fn deploy_site(
    username: &str,
    archive_path: &PathBuf,
) -> Result<SiteManifest, std::io::Error> {
    let site_id = uuid::Uuid::new_v4().to_string();
    let site_path = format!("data/{}", site_id);
    let archive_file = std::fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(archive_file)?;
    let manifest_content = {
        let mut manifest_file = archive.by_name("manifest.json")?;
        let mut content = String::new();
        manifest_file.read_to_string(&mut content)?;
        content
    };
    let mut manifest: SiteManifest = serde_json::from_str(&manifest_content)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => {
                if path.starts_with(format!("{}/", manifest.webroot)) {
                    let relative_path =
                        path.strip_prefix(format!("{}/", manifest.webroot)).unwrap();
                    std::path::Path::new(&format!("{}/webroot", site_path)).join(relative_path)
                } else {
                    continue;
                }
            }
            None => continue,
        };
        if file.is_file() {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            let mut perms = outfile.metadata()?.permissions();
            perms.set_mode(0o777);
            outfile.set_permissions(perms)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    manifest.site_id = Some(site_id);
    manifest.owner = Some(username.to_string());
    manifest.webroot = "webroot".to_string();
    manifest.deployed_at = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );
    let manifest_path = format!("{}/manifest.json", site_path);
    let manifest_json = serde_json::to_string(&manifest)?;
    tokio::fs::write(&manifest_path, manifest_json).await?;
    Ok(manifest)
}

pub async fn export_site(username: &str, site_id: &str) -> Result<PathBuf, std::io::Error> {
    let sites = list_sites(username).await?;
    if !sites
        .iter()
        .any(|site| site.site_id.as_deref() == Some(site_id))
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Site not found.",
        ));
    }
    let site_path = format!("data/{}", site_id);
    let archive_path = format!("/tmp/{}.zip", uuid::Uuid::new_v4().to_string());
    let cmd = format!(
        "cd \"{}\" && zip -r -o \"{}\" *",
        std::path::absolute(&site_path)?.to_string_lossy(),
        std::path::absolute(&archive_path)?.to_string_lossy(),
    );
    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()?;
    Ok(std::path::PathBuf::from(archive_path))
}

pub async fn delete_site(username: &str, site_id: &str) -> Result<(), std::io::Error> {
    let sites = list_sites(username).await?;
    if !sites
        .iter()
        .any(|site| site.site_id.as_deref() == Some(site_id))
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Site not found.",
        ));
    }
    let site_path = format!("data/{}", site_id);
    if tokio::fs::metadata(&site_path).await.is_ok() {
        tokio::fs::remove_dir_all(&site_path).await?;
    }
    Ok(())
}

pub async fn generate_site_template() -> Result<PathBuf, std::io::Error> {
    let (file, archive_path) = NamedTempFile::new()?.keep()?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::<()>::default();
    let manifest = SiteManifest {
        site_id: None,
        owner: None,
        webroot: "webroot".to_string(),
        deployed_at: None,
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(manifest_json.as_bytes())?;
    zip.add_directory("webroot/", options)?;
    zip.start_file("webroot/index.html", options)?;
    let index_html = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Hello, world!</title>
</head>
<body>
    <h1>Hello, world!</h1>
    <p>Hosted by 0Pages</p>
</body>
</html>"#;
    zip.write_all(index_html.as_bytes())?;
    zip.finish()?;
    Ok(archive_path)
}
