use crate::service::ssh_config::{
    export_ssh_config_to_file, import_ssh_config_from_file, load_ssh_servers,
};
use libadwaita::{
    ActionRow, ComboRow, PreferencesDialog, PreferencesGroup, PreferencesPage, SpinRow, SwitchRow,
    prelude::*,
};

pub fn create_preference_dialog() -> PreferencesDialog {
    let dialog = PreferencesDialog::builder()
        .title("Paramètres")
        .search_enabled(true)
        .build();

    let general_page = PreferencesPage::builder()
        .title("Général")
        .icon_name("preferences-system")
        .build();

    let general_group = PreferencesGroup::builder().title("Comportement").build();

    let auto_connect = SwitchRow::builder()
        .title("Connexion automatique")
        .subtitle("Se connecter automatiquement au dernier serveur utilisé")
        .sensitive(false)
        .build();

    let remember_servers = SwitchRow::builder()
        .title("Mémoriser les serveurs")
        .subtitle("Conserver l'historique des connexions")
        .sensitive(false)
        .build();

    let notifications = SwitchRow::builder()
        .title("Notifications")
        .subtitle("Afficher les notifications de connexion")
        .sensitive(false)
        .build();

    general_group.add(&auto_connect);
    general_group.add(&remember_servers);
    general_group.add(&notifications);
    general_page.add(&general_group);

    let interface_page = PreferencesPage::builder()
        .title("Interface")
        .icon_name("preferences-desktop")
        .build();

    let appearance_group = PreferencesGroup::builder().title("Apparence").build();

    let theme_combo = ComboRow::builder()
        .title("Thème")
        .subtitle("Choisir le thème de l'application")
        .sensitive(false)
        .build();

    let compact_mode = SwitchRow::builder()
        .title("Mode compact")
        .subtitle("Afficher plus d'éléments dans l'interface")
        .sensitive(false)
        .build();

    let show_tooltips = SwitchRow::builder()
        .title("Info-bulles")
        .subtitle("Afficher les informations au survol")
        .sensitive(false)
        .build();

    appearance_group.add(&theme_combo);
    appearance_group.add(&compact_mode);
    appearance_group.add(&show_tooltips);
    interface_page.add(&appearance_group);

    let ssh_page = PreferencesPage::builder()
        .title("SSH")
        .icon_name("network-server")
        .build();

    let connection_group = PreferencesGroup::builder().title("Connexion").build();

    let timeout_spin = SpinRow::builder()
        .title("Délai d'attente")
        .subtitle("Temps d'attente pour la connexion (secondes)")
        .sensitive(false)
        .build();

    let keep_alive = SwitchRow::builder()
        .title("Keep-alive")
        .subtitle("Maintenir la connexion active")
        .sensitive(false)
        .build();

    let compression = SwitchRow::builder()
        .title("Compression")
        .subtitle("Activer la compression des données")
        .sensitive(false)
        .build();

    connection_group.add(&timeout_spin);
    connection_group.add(&keep_alive);
    connection_group.add(&compression);
    ssh_page.add(&connection_group);

    let security_group = PreferencesGroup::builder().title("Sécurité").build();

    let strict_host_checking = SwitchRow::builder()
        .title("Vérification stricte des hôtes")
        .subtitle("Vérifier la clé d'hôte à chaque connexion")
        .sensitive(false)
        .build();

    let agent_forwarding = SwitchRow::builder()
        .title("Transfert d'agent")
        .subtitle("Permettre le transfert de l'agent SSH")
        .sensitive(false)
        .build();

    security_group.add(&strict_host_checking);
    security_group.add(&agent_forwarding);
    ssh_page.add(&security_group);

    let import_export_group = PreferencesGroup::builder().title("Import/Export").build();

    let import_row = ActionRow::builder()
        .title("Importer la configuration SSH")
        .subtitle("Remplacer la configuration actuelle par un fichier .ssh/config")
        .build();

    let import_button = gtk4::Button::builder()
        .label("Importer")
        .css_classes(vec!["destructive-action".to_string()])
        .build();

    import_button.connect_clicked(move |_| {
        import_ssh_config();
    });

    import_row.add_suffix(&import_button);

    let export_row = ActionRow::builder()
        .title("Exporter la configuration SSH")
        .subtitle("Sauvegarder la configuration vers un fichier .ssh/config")
        .build();

    let export_button = gtk4::Button::builder()
        .label("Exporter")
        .css_classes(vec!["suggested-action".to_string()])
        .build();

    export_button.connect_clicked(move |_| {
        export_ssh_config();
    });

    export_row.add_suffix(&export_button);

    import_export_group.add(&import_row);
    import_export_group.add(&export_row);
    ssh_page.add(&import_export_group);

    let advanced_page = PreferencesPage::builder()
        .title("Avancé")
        .icon_name("preferences-system-symbolic")
        .build();

    let debug_group = PreferencesGroup::builder().title("Débogage").build();

    let verbose_logging = SwitchRow::builder()
        .title("Journalisation détaillée")
        .subtitle("Enregistrer des informations détaillées dans les logs")
        .sensitive(false)
        .build();

    let debug_mode = SwitchRow::builder()
        .title("Mode débogage")
        .subtitle("Afficher des informations de débogage")
        .sensitive(false)
        .build();

    debug_group.add(&verbose_logging);
    debug_group.add(&debug_mode);
    advanced_page.add(&debug_group);

    let experimental_group = PreferencesGroup::builder().title("Expérimental").build();

    let experimental_features = SwitchRow::builder()
        .title("Fonctionnalités expérimentales")
        .subtitle("Activer les fonctionnalités en cours de développement")
        .sensitive(false)
        .build();

    experimental_group.add(&experimental_features);
    advanced_page.add(&experimental_group);

    dialog.add(&general_page);
    dialog.add(&interface_page);
    dialog.add(&ssh_page);
    dialog.add(&advanced_page);

    dialog
}

fn export_ssh_config() {
    let servers = match load_ssh_servers() {
        Ok(servers) => servers,
        Err(e) => {
            eprintln!("Erreur lors du chargement des serveurs SSH: {}", e);
            return;
        }
    };

    if servers.is_empty() {
        let info_dialog = gtk4::MessageDialog::builder()
            .modal(true)
            .message_type(gtk4::MessageType::Info)
            .text("Aucune configuration SSH")
            .secondary_text("Aucun serveur SSH n'a été trouvé dans la configuration actuelle.")
            .build();

        info_dialog.add_button("OK", gtk4::ResponseType::Ok);
        info_dialog.connect_response(|dialog, _| {
            dialog.close();
        });
        info_dialog.show();
        return;
    }

    let file_dialog = gtk4::FileChooserDialog::builder()
        .modal(true)
        .title("Exporter la configuration SSH")
        .action(gtk4::FileChooserAction::Save)
        .build();

    file_dialog.add_button("Annuler", gtk4::ResponseType::Cancel);
    file_dialog.add_button("Sauvegarder", gtk4::ResponseType::Accept);

    let filter = gtk4::FileFilter::new();
    filter.set_name(Some("Fichiers de configuration SSH"));
    filter.add_pattern("config");
    filter.add_pattern("*.config");
    file_dialog.add_filter(&filter);

    let all_filter = gtk4::FileFilter::new();
    all_filter.set_name(Some("Tous les fichiers"));
    all_filter.add_pattern("*");
    file_dialog.add_filter(&all_filter);

    file_dialog.set_current_name("ssh_config");

    file_dialog.connect_response(move |dialog, response| {
        if response == gtk4::ResponseType::Accept {
            if let Some(file_path) = dialog.file() {
                if let Some(path_str) = file_path.path() {
                    if let Some(path_str) = path_str.to_str() {
                        match export_ssh_config_to_file(&servers, path_str) {
                            Ok(_) => {
                                let success_dialog = gtk4::MessageDialog::builder()
                                    .transient_for(dialog)
                                    .modal(true)
                                    .message_type(gtk4::MessageType::Info)
                                    .text("Exportation réussie")
                                    .secondary_text(&format!(
                                        "La configuration SSH a été exportée vers:\n{}",
                                        path_str
                                    ))
                                    .build();

                                success_dialog.add_button("OK", gtk4::ResponseType::Ok);
                                success_dialog.connect_response(|dialog, _| {
                                    dialog.close();
                                });
                                success_dialog.show();
                            }
                            Err(e) => {
                                let error_dialog = gtk4::MessageDialog::builder()
                                    .transient_for(dialog)
                                    .modal(true)
                                    .message_type(gtk4::MessageType::Error)
                                    .text("Erreur d'exportation")
                                    .secondary_text(&format!(
                                        "Impossible d'exporter la configuration:\n{}",
                                        e
                                    ))
                                    .build();

                                error_dialog.add_button("OK", gtk4::ResponseType::Ok);
                                error_dialog.connect_response(|dialog, _| {
                                    dialog.close();
                                });
                                error_dialog.show();
                            }
                        }
                    }
                }
            }
        }
        dialog.close();
    });

    file_dialog.show();
}

fn import_ssh_config() {
    // Dialogue de confirmation avant l'import
    let confirm_dialog = gtk4::MessageDialog::builder()
        .modal(true)
        .message_type(gtk4::MessageType::Warning)
        .text("Confirmer l'importation")
        .secondary_text("Cette action va remplacer complètement votre configuration SSH actuelle. Une sauvegarde sera créée automatiquement. Voulez-vous continuer ?")
        .build();

    confirm_dialog.add_button("Annuler", gtk4::ResponseType::Cancel);
    confirm_dialog.add_button("Continuer", gtk4::ResponseType::Accept);

    confirm_dialog.connect_response(move |dialog, response| {
        if response == gtk4::ResponseType::Accept {
            let file_dialog = gtk4::FileChooserDialog::builder()
                .modal(true)
                .title("Importer la configuration SSH")
                .action(gtk4::FileChooserAction::Open)
                .build();

            file_dialog.add_button("Annuler", gtk4::ResponseType::Cancel);
            file_dialog.add_button("Importer", gtk4::ResponseType::Accept);

            let filter = gtk4::FileFilter::new();
            filter.set_name(Some("Fichiers de configuration SSH"));
            filter.add_pattern("config");
            filter.add_pattern("*.config");
            filter.add_pattern("*.ssh");
            file_dialog.add_filter(&filter);

            let all_filter = gtk4::FileFilter::new();
            all_filter.set_name(Some("Tous les fichiers"));
            all_filter.add_pattern("*");
            file_dialog.add_filter(&all_filter);

            file_dialog.connect_response(move |dialog, response| {
                if response == gtk4::ResponseType::Accept {
                    if let Some(file_path) = dialog.file() {
                        if let Some(path_str) = file_path.path() {
                            if let Some(path_str) = path_str.to_str() {
                                match import_ssh_config_from_file(path_str) {
                                    Ok(_) => {
                                        let success_dialog = gtk4::MessageDialog::builder()
                                            .transient_for(dialog)
                                            .modal(true)
                                            .message_type(gtk4::MessageType::Info)
                                            .text("Importation réussie")
                                            .secondary_text(&format!(
                                                "La configuration SSH a été importée depuis:\n{}\n\nUne sauvegarde de l'ancienne configuration a été créée dans ~/.ssh/config.backup",
                                                path_str
                                            ))
                                            .build();

                                        success_dialog.add_button("OK", gtk4::ResponseType::Ok);
                                        success_dialog.connect_response(|dialog, _| {
                                            dialog.close();
                                        });
                                        success_dialog.show();
                                    }
                                    Err(e) => {
                                        let error_dialog = gtk4::MessageDialog::builder()
                                            .transient_for(dialog)
                                            .modal(true)
                                            .message_type(gtk4::MessageType::Error)
                                            .text("Erreur d'importation")
                                            .secondary_text(&format!(
                                                "Impossible d'importer la configuration:\n{}",
                                                e
                                            ))
                                            .build();

                                        error_dialog.add_button("OK", gtk4::ResponseType::Ok);
                                        error_dialog.connect_response(|dialog, _| {
                                            dialog.close();
                                        });
                                        error_dialog.show();
                                    }
                                }
                            }
                        }
                    }
                }
                dialog.close();
            });

            file_dialog.show();
        }
        dialog.close();
    });

    confirm_dialog.show();
}
