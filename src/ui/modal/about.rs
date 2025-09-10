use gtk4::License;
use libadwaita::AboutDialog;

pub fn create_about_dialog() -> AboutDialog {
    AboutDialog::builder()
        .application_name("SSH Config Manager")
        .application_icon("network-server")
        .version(env!("CARGO_PKG_VERSION"))
        .copyright("© 2025 Cleboost")
        .license_type(License::MitX11)
        .license("MIT License")
        .developer_name("Cleboost")
        .issue_url("https://github.com/Cleboost/SSH-Config-Manager/issues")
        .website("https://github.com/Cleboost/SSH-Config-Manager")
        .build()
}
