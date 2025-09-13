use crate::service::delete_ssh_server;
use libadwaita::prelude::*;
use libadwaita::{AlertDialog, ResponseAppearance};
use std::rc::Rc;

pub fn create_delete_server_dialog(
    server_name: &str,
    on_delete: Option<Rc<dyn Fn() + 'static>>,
) -> AlertDialog {
    let dialog = AlertDialog::builder()
        .heading("Delete Server")
        .body(&format!(
            "Are you sure you want to delete the server \"{}\" ?\n\nThis action is irreversible.",
            server_name
        ))
        .build();

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("delete", "Delete");

    dialog.set_response_appearance("delete", ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    let server_name = server_name.to_string();
    dialog.connect_response(None, move |_, response| {
        if response == "delete" {
            match delete_ssh_server(&server_name) {
                Ok(_) => {
                    if let Some(refresh_fn) = &on_delete {
                        refresh_fn();
                    }
                }
                Err(e) => eprintln!("Error deleting server '{}': {}", server_name, e),
            }
        }
    });

    dialog
}
