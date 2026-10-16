//! Product vocabulary, shared by visible text and native accessible names.
pub const CODES: [&str; 4] = ["it", "fr", "es", "de"];
include!("i18n_locales.rs");
pub const NAMES: [&str; 4] = ["Italiano", "Français", "Español", "Deutsch"];

macro_rules! messages {
    ($($key:ident => [$it:literal, $fr:literal, $es:literal, $de:literal, $en:literal, $sq:literal]),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        // Shared vocabulary also includes keys consumed only by the React settings window.
        #[allow(dead_code)]
        pub enum Text { $($key),+ }
        impl Text {
            pub fn key(self) -> &'static str { match self { $(Self::$key => stringify!($key)),+ } }
        }
        #[cfg(test)]
        pub const ALL_TEXT: &[Text] = &[$(Text::$key),+];

    };
}
type Dictionaries = std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>>;
fn dictionaries() -> &'static Dictionaries {
    static DATA: std::sync::OnceLock<Dictionaries> = std::sync::OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../../../localization/messages.json"))
            .expect("validated translation dictionaries")
    })
}
pub fn tr(language: &str, key: Text) -> &'static str {
    dictionaries()
        .get(language)
        .unwrap_or_else(|| &dictionaries()["it"])
        .get(key.key())
        .expect("complete translation dictionary")
        .as_str()
}

messages! {
    KeepEditing => ["Continua a modificare", "Continuer la modification", "Seguir editando", "Weiter bearbeiten", "Keep editing", "Vazhdo redaktimin"],
    Settings => ["Impostazioni", "Paramètres", "Ajustes", "Einstellungen", "Settings", "Cilësimet"],
    General => ["Generale", "Général", "General", "Allgemein", "General", "Të përgjithshme"],
    Sets => ["Set caratteri", "Jeux de caractères", "Conjuntos", "Zeichensätze", "Character sets", "Grupet e shkronjave"],
    Blacklist => ["BlackList", "BlackList", "BlackList", "BlackList", "Excluded applications", "Aplikacionet e përjashtuara"],
    GeneralHint => ["Scrittura e lingua. Tutto al suo posto.", "Écriture et langue, au même endroit.", "Escritura e idioma, en un solo lugar.", "Schreiben und Sprache an einem Ort.", "Writing and language. All in one place.", "Shkrimi dhe gjuha. Të gjitha në një vend."],
    SetsHint => ["Scegli i caratteri da avere a portata di mano.", "Choisissez les caractères à portée de main.", "Elige los caracteres que quieres tener a mano.", "Wähle die Zeichen, die du griffbereit brauchst.", "Choose the characters you want at hand.", "Zgjidhni shkronjat që dëshironi të keni pranë."],
    BlacklistHint => ["Le applicazioni in cui Accento resta spento.", "Les applications dans lesquelles Accento reste inactif.", "Las aplicaciones en las que Accento permanece inactivo.", "Anwendungen, in denen Accento inaktiv bleibt.", "Applications where Accento stays off.", "Aplikacionet ku Accento qëndron i çaktivizuar."],
    Delay => ["Ritardo di apertura", "Délai d’ouverture", "Retraso de apertura", "Öffnungsverzögerung", "Opening delay", "Vonesa e hapjes"],
    DelayHint => ["Da 100 a 1500 ms", "De 100 à 1500 ms", "De 100 a 1500 ms", "100 bis 1500 ms", "From 100 to 1500 ms", "Nga 100 deri në 1500 ms"],
    Less => ["Riduci ritardo", "Réduire le délai", "Reducir retraso", "Verzögerung verkürzen", "Decrease delay", "Zvogëloni vonesën"],
    More => ["Aumenta ritardo", "Augmenter le délai", "Aumentar retraso", "Verzögerung verlängern", "Increase delay", "Rritni vonesën"],
    ActiveSets => ["Set caratteri attivi", "Jeux de caractères actifs", "Conjuntos activos", "Aktive Zeichensätze", "Active character sets", "Grupet aktive të shkronjave"],
    SetDetail => ["Vedi tutti i caratteri →", "Voir tous les caractères →", "Ver todos los caracteres →", "Alle Zeichen ansehen →", "See all characters →", "Shikoni të gjitha shkronjat →"],
    Appearance => ["Aspetto", "Apparence", "Aspecto", "Darstellung", "Appearance", "Pamja"],
    System => ["Sistema", "Système", "Sistema", "System", "System", "Sistemi"],
    Light => ["Chiaro", "Clair", "Claro", "Hell", "Light", "E çelët"],
    Dark => ["Scuro", "Sombre", "Oscuro", "Dunkel", "Dark", "E errët"],
    UiLanguage => ["Lingua dell’interfaccia", "Langue de l’interface", "Idioma de la interfaz", "Sprache der Oberfläche", "Interface language", "Gjuha e ndërfaqes"],
    Independent => ["Indipendente dai set caratteri attivi.", "Indépendante des jeux de caractères actifs.", "Independiente de los conjuntos activos.", "Unabhängig von den aktiven Zeichensätzen.", "Independent of active character sets.", "E pavarur nga grupet aktive të shkronjave."],
    Gesture => ["Lettera + Spazio: il primo carattere è già selezionato.", "Lettre + Espace : le premier caractère est sélectionné.", "Letra + Espacio: el primer carácter ya está seleccionado.", "Buchstabe + Leertaste: Das erste Zeichen ist ausgewählt.", "Letter + Space: the first character is already selected.", "Shkronjë + Hapësirë: shkronja e parë është zgjedhur."],
    Release => ["← → o Spazio per cambiare · Rilascia la lettera per inserire · Esc annulla", "← → ou Espace pour changer · Relâchez pour insérer · Échap annule", "← → o Espacio para cambiar · Suelta para insertar · Esc cancela", "← → oder Leertaste zum Wechseln · Loslassen zum Einfügen · Esc bricht ab", "← → or Space to change · Release the letter to insert · Esc cancels", "← → ose Hapësirë për të ndryshuar · Lëshoni shkronjën për ta futur · Esc anulon"],
    SetsNote => ["Puoi attivare più set. I caratteri comuni compaiono una sola volta.", "Plusieurs jeux possibles. Les caractères communs ne sont pas répétés.", "Puedes activar varios conjuntos. Los caracteres comunes no se repiten.", "Mehrere Sätze möglich. Gemeinsame Zeichen erscheinen nur einmal.", "Enable multiple sets. Shared characters appear only once.", "Mund të aktivizoni disa grupe. Shkronjat e përbashkëta shfaqen vetëm një herë."],
    AtLeastOne => ["Mantieni almeno un set attivo.", "Gardez au moins un jeu actif.", "Mantén al menos un conjunto activo.", "Mindestens ein Zeichensatz muss aktiv bleiben.", "Keep at least one character set active.", "Mbani të paktën një grup aktiv."],
    AddApp => ["Aggiungi applicazione", "Ajouter une application", "Añadir aplicación", "Anwendung hinzufügen", "Add application", "Shtoni aplikacion"],
    Add => ["Aggiungi", "Ajouter", "Añadir", "Hinzufügen", "Add", "Shto"],
    AppHint => ["Un nome per volta, per esempio notepad.exe", "Un nom à la fois, par exemple notepad.exe", "Un nombre cada vez, por ejemplo notepad.exe", "Jeweils ein Name, zum Beispiel notepad.exe", "One name at a time, for example notepad.exe", "Një emër çdo herë, për shembull notepad.exe"],
    AppPlaceholder => ["nome-app.exe", "application.exe", "aplicacion.exe", "anwendung.exe", "application.exe", "aplikacioni.exe"],
    Empty => ["Nessuna applicazione nella BlackList", "Aucune application dans la BlackList", "Ninguna aplicación en la BlackList", "Keine Anwendung in der BlackList", "No excluded applications", "Asnjë aplikacion i përjashtuar"],
    EmptyHint => ["Accento è disponibile in tutte le applicazioni.", "Accento est disponible dans toutes les applications.", "Accento está disponible en todas las aplicaciones.", "Accento ist in allen Anwendungen verfügbar.", "Accento is available in all applications.", "Accento është i disponueshëm në të gjitha aplikacionet."],
    Remove => ["Rimuovi", "Supprimer", "Eliminar", "Entfernen", "Remove", "Hiq"],
    Save => ["Salva modifiche", "Enregistrer", "Guardar cambios", "Speichern", "Save changes", "Ruaj ndryshimet"],
    Saved => ["Tutte le modifiche sono salvate", "Toutes les modifications sont enregistrées", "Todos los cambios están guardados", "Alle Änderungen sind gespeichert", "All changes are saved", "Të gjitha ndryshimet janë ruajtur"],
    CheckField => ["Controlla il campo evidenziato", "Vérifiez le champ indiqué", "Revisa el campo indicado", "Prüfe das markierte Feld", "Check the highlighted field", "Kontrolloni fushën e shënuar"],
    Pending => ["Modifiche da salvare", "Modifications à enregistrer", "Cambios sin guardar", "Ungespeicherte Änderungen", "Unsaved changes", "Ndryshime të paruajtura"],
    Enabled => ["Accento attivo", "Accento actif", "Accento activo", "Accento aktiv", "Accento is on", "Accento është aktiv"],
    Disabled => ["Accento spento", "Accento désactivé", "Accento desactivado", "Accento ausgeschaltet", "Accento is off", "Accento është joaktiv"],
    TurnOn => ["Attiva Accento", "Activer Accento", "Activar Accento", "Accento einschalten", "Turn Accento on", "Aktivizo Accento"],
    TurnOff => ["Spegni Accento", "Désactiver Accento", "Desactivar Accento", "Accento ausschalten", "Turn Accento off", "Çaktivizo Accento"],
    Pause => ["Pausa", "Pause", "Pausa", "Pausieren", "Pause", "Pauzë"],
    Resume => ["Riprendi", "Reprendre", "Reanudar", "Fortsetzen", "Resume", "Vazhdo"],
    Exit => ["Esci", "Quitter", "Salir", "Beenden", "Exit", "Dil"],
    Selected => ["selezionato", "sélectionné", "seleccionado", "ausgewählt", "selected", "zgjedhur"],
    NotSelected => ["non selezionato", "non sélectionné", "no seleccionado", "nicht ausgewählt", "not selected", "pa zgjedhur"],
    DelayError => ["Inserisci un ritardo tra 100 e 1500 ms.", "Saisissez un délai de 100 à 1500 ms.", "Introduce un retraso de 100 a 1500 ms.", "Gib eine Verzögerung von 100 bis 1500 ms ein.", "Enter a delay between 100 and 1500 ms.", "Vendosni një vonesë nga 100 deri në 1500 ms."],
    AppError => ["Inserisci un solo nome .exe, senza percorso o virgole.", "Saisissez un seul nom .exe, sans chemin ni virgule.", "Introduce un solo nombre .exe, sin ruta ni comas.", "Gib einen einzelnen .exe-Namen ohne Pfad oder Kommas ein.", "Enter a single .exe name, without a path or commas.", "Vendosni një emër të vetëm .exe, pa shteg ose presje."],
    Duplicate => ["Questa applicazione è già nella BlackList.", "Cette application figure déjà dans la BlackList.", "Esta aplicación ya está en la BlackList.", "Diese Anwendung steht bereits in der BlackList.", "This application is already excluded.", "Ky aplikacion është përjashtuar tashmë."],
    PickerError => ["Selettore non disponibile. Accento è spento.", "Sélecteur indisponible. Accento est désactivé.", "Selector no disponible. Accento está desactivado.", "Auswahl nicht verfügbar. Accento ist ausgeschaltet.", "Character picker unavailable. Accento is off.", "Zgjedhësi i shkronjave nuk është i disponueshëm. Accento është joaktiv."],
    ConfigError => ["Configurazione non supportata.", "Configuration non prise en charge.", "Configuración no compatible.", "Konfiguration wird nicht unterstützt.", "Unsupported configuration.", "Konfigurim i pambështetur."],
    LanguageError => ["Scegli una lingua disponibile.", "Choisissez une langue disponible.", "Elige un idioma disponible.", "Wähle eine verfügbare Sprache.", "Choose an available language.", "Zgjidhni një gjuhë të disponueshme."],
    Variants => ["Varianti", "Variantes", "Variantes", "Varianten", "Variants", "Variantet"],
    InsertHint => ["Rilascia la lettera per inserire", "Relâchez la lettre pour insérer", "Suelta la letra para insertar", "Buchstabe loslassen zum Einfügen", "Release the letter to insert", "Lëshoni shkronjën për ta futur"],
    Discard => ["Annulla modifiche", "Annuler les modifications", "Descartar cambios", "Änderungen verwerfen", "Discard changes", "Anulo ndryshimet"],
    SearchLanguage => ["Cerca lingua", "Rechercher une langue", "Buscar idioma", "Sprache suchen", "Search languages", "Kërko gjuhën"],
    NoLanguage => ["Nessuna lingua trovata", "Aucune langue trouvée", "No se encontraron idiomas", "Keine Sprache gefunden", "No languages found", "Nuk u gjet asnjë gjuhë"],
    ClearSearch => ["Cancella ricerca", "Effacer la recherche", "Borrar búsqueda", "Suche löschen", "Clear search", "Pastro kërkimin"],
    Loading => ["Caricamento…", "Chargement…", "Cargando…", "Wird geladen…", "Loading…", "Duke u ngarkuar…"],
    Retry => ["Riprova", "Réessayer", "Reintentar", "Erneut versuchen", "Try again", "Provo përsëri"],
    Writing => ["Scrittura", "Écriture", "Escritura", "Schreiben", "Writing", "Shkrimi"],
    QuickGuide => ["Come si usa", "Mode d’emploi", "Cómo se usa", "Bedienung", "How it works", "Si përdoret"],
    CloseDraft => ["Ci sono modifiche non salvate. Chiudere e perderle?", "Des modifications ne sont pas enregistrées. Fermer et les perdre ?", "Hay cambios sin guardar. ¿Cerrar y perderlos?", "Ungespeicherte Änderungen. Schließen und verwerfen?", "You have unsaved changes. Close and discard them?", "Keni ndryshime të paruajtura. Të mbyllen pa i ruajtur?"],
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_national_locale_has_explicit_native_and_web_translations() {
        for locale in UI_CODES {
            let messages = dictionaries().get(locale).expect("missing locale");
            for key in ALL_TEXT {
                assert!(
                    !messages[key.key()].trim().is_empty(),
                    "{locale}: {}",
                    key.key()
                );
            }
            assert!(
                messages.contains_key("GuideTitle"),
                "{locale}: missing guide"
            );
        }
        assert_eq!(dictionaries().len(), UI_CODES.len());
    }
    #[test]
    fn supported_languages_have_independent_ui_labels() {
        assert_eq!(tr("en", Text::Settings), "Settings");
        assert_eq!(tr("sq", Text::UiLanguage), "Gjuha e ndërfaqes");
        assert_eq!(tr("de", Text::General), "Allgemein");
        assert_eq!(tr("fr", Text::Save), "Enregistrer");
        assert_eq!(tr("es", Text::UiLanguage), "Idioma de la interfaz");
        assert_eq!(tr("unknown", Text::General), tr("it", Text::General));
    }
}
