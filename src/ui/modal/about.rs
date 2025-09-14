use gtk4::License;
use libadwaita::AboutDialog;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Deserialize)]
struct Release {
    body: Option<String>,
}

static RELEASE_NOTES: OnceLock<String> = OnceLock::new();

fn fetch_release_notes() -> Option<Release> {
    let url = "https://api.github.com/repos/Cleboost/Rustmius/releases/latest";

    let client = reqwest::blocking::Client::builder()
        .user_agent("Rustmius/0.2.0")
        .timeout(std::time::Duration::from_secs(10))
        .build();

    match client {
        Ok(client) => match client.get(url).send() {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Release>() {
                        Ok(release) => Some(release),
                        Err(e) => {
                            eprintln!("Erreur parsing JSON release: {}", e);
                            None
                        }
                    }
                } else {
                    eprintln!("Erreur HTTP: {}", response.status());
                    None
                }
            }
            Err(e) => {
                eprintln!("Erreur requête GitHub: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("Erreur création client HTTP: {}", e);
            None
        }
    }
}

fn format_release_notes(release: &Release) -> String {
    let mut html = String::new();

    // Version + date en un seul paragraphe
    /*html.push_str("<p>Version ");
    html.push_str(&release.tag_name);
    html.push_str(" — ");
    html.push_str(&release.created_at[..10]);
    html.push_str("</p>");*/

    if let Some(body) = &release.body {
        let mut in_list = false;
        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("##") || trimmed.starts_with("**") {
                if in_list {
                    html.push_str("</ul>");
                    in_list = false;
                }
                continue;
            }

            if trimmed.starts_with('*') || trimmed.starts_with('-') {
                if !in_list {
                    html.push_str("<ul>");
                    in_list = true;
                }
                // Supprimer le '*' ou '-' initial
                let mut item = trimmed[1..].trim().to_string();

                // Mettre les mentions @ en <code> pour ressortir
                item = item
                    .split_whitespace()
                    .map(|word| {
                        if word.starts_with('@') {
                            format!("<code>{}</code>", word)
                        } else {
                            word.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                html.push_str("<li>");
                html.push_str(&item);
                html.push_str("</li>");
            } else {
                if in_list {
                    html.push_str("</ul>");

                    in_list = false;
                }
                html.push_str("<p>");
                html.push_str(trimmed);
                html.push_str("</p>");
            }
        }
        if in_list {
            html.push_str("</ul>");
        }
    }

    html
}

fn get_static_release_notes() -> String {
    match env!("CARGO_PKG_VERSION") {
        "0.2.0" => {
            "<p><strong>Version :</strong> v0.2.0</p><p><strong>Date :</strong> 2025-09-13</p><p><strong>Changements :</strong></p><ul><li>Configure Renovate by @renovate[bot]</li><li>Add export SSH key modal and improve server dialog</li><li>Add server deletion functionality to UI</li><li>New contributors: @renovate[bot] and @Cleboost</li></ul>".to_string()
        }
        _ => format!("Release notes for version {} are not available.", env!("CARGO_PKG_VERSION"))
    }
}

fn get_release_notes() -> &'static str {
    RELEASE_NOTES.get_or_init(|| match fetch_release_notes() {
        Some(release) => format_release_notes(&release),
        None => {
            eprintln!("Falling back to static release notes");
            get_static_release_notes()
        }
    })
}

pub fn create_about_dialog() -> AboutDialog {
    AboutDialog::builder()
        .application_name("SSH Config Manager")
        .application_icon("network-server")
        .version(env!("CARGO_PKG_VERSION"))
        .copyright("© 2025 Cleboost")
        .license_type(License::MitX11)
        .license("MIT License")
        .developer_name("Cleboost")
        .issue_url("https://github.com/Cleboost/Rustmius/issues")
        .website("https://github.com/Cleboost/Rustmius")
        .release_notes(get_release_notes())
        .release_notes_version(env!("CARGO_PKG_VERSION"))
        .build()
}
