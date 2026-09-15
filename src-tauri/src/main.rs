// Empêche l'ouverture d'une console Windows en plus de la fenêtre applicative.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    asgard_crm_lib::run()
}
