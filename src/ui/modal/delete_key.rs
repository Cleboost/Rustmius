use libadwaita::prelude::*;
use libadwaita::{AlertDialog, ResponseAppearance};

/// Crée un dialog de confirmation de suppression de clé SSH
pub fn create_delete_key_dialog(key_name: &str, key_type: &str) -> AlertDialog {
    let dialog = AlertDialog::builder()
        .heading("Supprimer la clé SSH")
        .body(&format!("Êtes-vous sûr de vouloir supprimer la clé SSH \"{}\" ({}) ?\n\nCette action est irréversible.", key_name, key_type))
        .build();

    // Ajouter les boutons de réponse
    dialog.add_response("cancel", "Annuler");
    dialog.add_response("delete", "Supprimer");

    // Configurer l'apparence des boutons
    dialog.set_response_appearance("delete", ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    dialog
}
