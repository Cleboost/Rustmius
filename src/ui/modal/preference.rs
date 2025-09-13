use libadwaita::{prelude::*, PreferencesDialog, PreferencesGroup, PreferencesPage, SwitchRow};

pub fn create_preference_dialog() -> PreferencesDialog {
    let dialog = PreferencesDialog::builder()
        .title("Paramètres")
        .search_enabled(true)
        .build();

    let page = PreferencesPage::builder().build();

    let test = SwitchRow::builder().title("Test").build();

    let group = PreferencesGroup::builder().title("Test Group").build();
    group.add(&test);

    page.add(&group);

    dialog.add(&page);
    dialog
}
