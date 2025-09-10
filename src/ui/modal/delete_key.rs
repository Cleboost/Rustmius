use libadwaita::prelude::*;
use libadwaita::{AlertDialog, ResponseAppearance};

pub fn create_delete_key_dialog(key_name: &str, key_type: &str) -> AlertDialog {
    let dialog = AlertDialog::builder()
        .heading("Supprimer la clé SSH")
        .body(&format!("Êtes-vous sûr de vouloir supprimer la clé SSH \"{}\" ({}) ?\n\nCette action est irréversible.", key_name, key_type))
        .build();

    dialog.add_response("cancel", "Annuler");
    dialog.add_response("delete", "Supprimer");

    dialog.set_response_appearance("delete", ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    dialog
}
