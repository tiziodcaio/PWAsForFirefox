use anyhow::Result;
use cfg_if::cfg_if;
use log::warn;
use web_app_manifest::resources::{IconResource, ShortcutResource};
use web_app_manifest::types::Url;

use crate::components::site::Site;
use crate::directories::ProjectDirs;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

const INVALID_URL: &str = "Site without valid absolute URL is not possible";

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SiteInfoInstall<'a> {
    id: String,
    url: String,
    domain: String,
    name: String,
    description: String,
    icons: &'a Vec<IconResource>,
    shortcuts: &'a Vec<ShortcutResource>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SiteInfoUninstall {
    id: String,
    name: String,
}

pub fn install(site: &Site, dirs: &ProjectDirs) -> Result<()> {
    // Site ID is used to generate all commands and as an identifier at various places
    let id = site.ulid.to_string();

    // Start URL is used as an info URL in ARP
    // First try the user-specified start URL, then try manifest start URL
    let url = if let Some(url) = &site.config.start_url {
        url.to_string()
    } else if let Url::Absolute(url) = &site.manifest.start_url {
        url.to_string()
    } else {
        unreachable!(INVALID_URL)
    };

    // Scope domain is used as a publisher name or when the site name is undefined
    // Using scope instead of start URL because user should not be able to overwrite it
    let domain = if let Url::Absolute(url) = &site.manifest.scope {
        if let Some(domain) = url.host() {
            domain.to_string()
        } else {
            unreachable!(INVALID_URL)
        }
    } else {
        unreachable!(INVALID_URL)
    };

    // First try the user-specified name, then try manifest names, then fall back to domain name
    let name = if let Some(name) = &site.config.name {
        name.clone()
    } else if let Some(name) = &site.manifest.name {
        name.clone()
    } else if let Some(name) = &site.manifest.short_name {
        name.clone()
    } else {
        domain.clone()
    };

    // First try the user-specified name, then try manifest description
    let description = if let Some(description) = &site.config.description {
        description.clone()
    } else if let Some(description) = &site.manifest.description {
        description.clone()
    } else {
        "".into()
    };

    // Generate site info struct
    let info = SiteInfoInstall {
        id,
        url,
        domain,
        name,
        description,
        icons: &site.manifest.icons,
        shortcuts: &site.manifest.shortcuts,
    };

    // Apply system integration
    cfg_if! {
        if #[cfg(target_os = "windows")] {
            windows::install(&info, &dirs)
        } else if #[cfg(target_os = "linux")] {
            linux::install(&info, &dirs)
        } else if #[cfg(target_os = "macos")] {
            warn!("System integration currently does not work on macOS");
            Ok(())
        } else {
            compile_error!("Unknown operating system");
        }
    }
}

pub fn uninstall(site: &Site, dirs: &ProjectDirs) -> Result<()> {
    // Site ID is used to generate all commands and as an identifier at various places
    let id = site.ulid.to_string();

    // Scope domain is used as a publisher name or when the site name is undefined
    // Using scope instead of start URL because user should not be able to overwrite it
    let domain = if let Url::Absolute(url) = &site.manifest.scope {
        if let Some(domain) = url.host() {
            domain.to_string()
        } else {
            unreachable!(INVALID_URL)
        }
    } else {
        unreachable!(INVALID_URL)
    };

    // First try the user-specified name, then try manifest names, then fall back to domain name
    let name = if let Some(name) = &site.config.name {
        name.clone()
    } else if let Some(name) = &site.manifest.name {
        name.clone()
    } else if let Some(name) = &site.manifest.short_name {
        name.clone()
    } else {
        domain
    };

    // Generate site info struct
    let info = SiteInfoUninstall { id, name };

    // Unapply system integration
    cfg_if! {
        if #[cfg(target_os = "windows")] {
            windows::uninstall(&info, &dirs)
        } else if #[cfg(target_os = "linux")] {
            linux::uninstall(&info, &dirs)
        } else if #[cfg(target_os = "macos")] {
            warn!("System integration currently does not work on macOS");
            Ok(())
        } else {
            compile_error!("Unknown operating system");
        }
    }
}
