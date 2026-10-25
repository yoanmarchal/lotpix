// Fichiers des menus contextuels, produits à partir des préréglages (src/presets.rs) et des
// traductions (i18n/) : le bloc NSIS de l'installeur Windows dans Cargo.toml, les menus de
// Dolphin et les actions de Nemo. Le test échoue s'ils ne sont plus à jour ;
// `LOTPIX_UPDATE_MENUS=1 cargo test` les réécrit
use std::fs;
use std::path::{Path, PathBuf};

use crate::i18n::{self, Language};
use crate::presets::Submenu;
use crate::t;

/// Langues de l'installeur, avec leur nom dans NSIS (`${LANG_…}`).
const LANGUAGES: [(Language, &str); 4] = [
    (Language::En, "ENGLISH"),
    (Language::Fr, "FRENCH"),
    (Language::Es, "SPANISH"),
    (Language::De, "GERMAN"),
];

const MIME_TYPES: &str = "image/jpeg;image/png;image/gif;image/tiff;image/webp;inode/directory;";

const GENERATED: &str = "Généré par `LOTPIX_UPDATE_MENUS=1 cargo test` (src/context_menu.rs) : ne pas modifier à la main";
const NSIS_START: &str = "# Début du menu contextuel de l'Explorateur.";
const NSIS_END: &str = "# Fin du menu contextuel de l'Explorateur";

/// Texte dans chaque langue, l'anglais en premier.
fn translations(text: impl Fn() -> String) -> Vec<(Language, String)> {
    let previous = i18n::current();
    let texts = LANGUAGES
        .iter()
        .map(|(language, _)| {
            i18n::set_language(*language);
            (*language, text())
        })
        .collect();
    i18n::set_language(previous);
    texts
}

/// Clé d'un fichier .desktop : la valeur anglaise, puis les traductions qui en diffèrent.
fn desktop_key(key: &str, text: impl Fn() -> String) -> String {
    let texts = translations(text);
    let english = &texts[0].1;
    let mut lines = vec![format!("{key}={english}")];
    for (language, value) in &texts[1..] {
        if value != english {
            lines.push(format!("{key}[{}]={value}", language.code()));
        }
    }
    lines.join("\n")
}

fn dolphin_submenu(submenu: Submenu) -> String {
    let entries = submenu.entries();
    let actions: String = entries
        .iter()
        .map(|(value, _)| format!("{value};"))
        .collect();
    let mut sections = vec![format!(
        "# {GENERATED}\n[Desktop Entry]\nType=Service\nMimeType={MIME_TYPES}\nActions={actions}\n{}\nX-KDE-Priority=TopLevel\nIcon=lotpix",
        desktop_key("X-KDE-Submenu", || submenu.title())
    )];
    for (index, (value, _)) in entries.iter().enumerate() {
        sections.push(format!(
            "[Desktop Action {value}]\n{}\nExec=lotpix {} {value} %F",
            desktop_key("Name", || submenu.entries()[index].1.clone()),
            submenu.option()
        ));
    }
    sections.join("\n\n") + "\n"
}

fn dolphin_open() -> String {
    format!(
        "# {GENERATED}\n[Desktop Entry]\nType=Service\nMimeType={MIME_TYPES}\nActions=open;\nX-KDE-Priority=TopLevel\n\n[Desktop Action open]\n{}\nIcon=lotpix\nExec=lotpix open %F\n",
        desktop_key("Name", || t!("menu-open"))
    )
}

/// Action Nemo. Nemo n'a pas de sous-menus : une entrée de sous-menu ouvre la liste de choix
/// du script lotpix-menu.sh, qui demande zenity.
fn nemo_action(
    name: impl Fn() -> String,
    comment: impl Fn() -> String,
    exec: &str,
    needs_zenity: bool,
) -> String {
    let mut lines = vec![
        format!("# {GENERATED}"),
        "[Nemo Action]".to_owned(),
        desktop_key("Name", name),
        desktop_key("Comment", comment),
        "Icon-Name=lotpix".to_owned(),
        format!("Exec={exec}"),
        "Selection=notnone".to_owned(),
        format!("Mimetypes={MIME_TYPES}"),
    ];
    if needs_zenity {
        lines.push("Dependencies=zenity;".to_owned());
    }
    lines.join("\n") + "\n"
}

/// Chaîne NSIS entre guillemets : `$` et `"` s'échappent.
fn nsis_string(text: &str) -> String {
    format!("\"{}\"", text.replace('$', "$$").replace('"', "$\\\""))
}

/// Bloc `preinstall-section` de Cargo.toml, entre ses deux commentaires de repère.
fn nsis_section() -> String {
    let mut strings: Vec<(String, Vec<(Language, String)>)> = vec![
        (
            "lotpix_compress".to_owned(),
            translations(|| t!("menu-compress")),
        ),
        (
            "lotpix_resize".to_owned(),
            translations(|| t!("menu-resize")),
        ),
        ("lotpix_open".to_owned(), translations(|| t!("menu-open"))),
    ];
    let mut menu = Vec::new();
    for (submenu, key, prefix) in [
        (Submenu::Compress, "Lotpix.Compress", "lotpix_compress"),
        (Submenu::Resize, "Lotpix.Resize", "lotpix_resize"),
    ] {
        let base = format!("Software\\Classes\\${{CLASS}}\\shell\\{key}");
        menu.push(format!(
            "  !insertmacro LotpixSubmenu \"{base}\" \"$({prefix})\""
        ));
        let entries = submenu.entries();
        for (index, (value, _)) in entries.iter().enumerate() {
            let name = format!("{prefix}_{}", value.replace('-', "_"));
            strings.push((
                name.clone(),
                translations(|| submenu.entries()[index].1.clone()),
            ));
            // Les entrées d'un sous-menu sont triées par nom de clé
            let entry_key = format!("{base}\\shell\\{:02}-{value}", index + 1);
            menu.push(format!(
                "  !insertmacro LotpixCommand \"{entry_key}\" \"$({name})\" \"{} {value}\" \"${{ARGUMENT}}\"",
                submenu.option()
            ));
            if submenu == Submenu::Resize && *value == "instagram-square" {
                menu.push(
                    "  ; Séparateur avant les réseaux sociaux (ECF_SEPARATORBEFORE)".to_owned(),
                );
                menu.push(format!(
                    "  WriteRegDWORD SHCTX \"{entry_key}\" \"CommandFlags\" 0x20"
                ));
            }
        }
        menu.push(String::new());
    }

    let lang_strings: Vec<String> = strings
        .iter()
        .flat_map(|(name, texts)| {
            texts.iter().map(move |(language, text)| {
                let nsis_language = LANGUAGES
                    .iter()
                    .find(|(other, _)| other == language)
                    .map(|(_, nsis)| nsis)
                    .expect("langue de l'installeur");
                format!(
                    "LangString {name} ${{LANG_{nsis_language}}} {}",
                    nsis_string(text)
                )
            })
        })
        .collect();

    format!(
        r#"{NSIS_START} {GENERATED}.
# « Compresser avec Lotpix » et « Redimensionner avec Lotpix » ouvrent un sous-menu, avec les
# réglages recommandés (src/presets.rs) ; « Ouvrir dans Lotpix » ouvre l'interface. Les libellés
# sont écrits dans la langue de l'installation. Les entrées classiques apparaissent sous
# « Afficher plus d'options » dans Windows 11. MultiSelectModel = Player lève la limite de
# 15 fichiers sélectionnés ; Windows lance alors un processus par fichier, que l'application
# regroupe (src/queue.rs)
preinstall-section = '''
!macro LotpixCommand KEY LABEL ARGUMENTS ARGUMENT
  WriteRegStr SHCTX "${{KEY}}" "MUIVerb" "${{LABEL}}"
  WriteRegStr SHCTX "${{KEY}}" "MultiSelectModel" "Player"
  WriteRegStr SHCTX "${{KEY}}\command" "" "$\"$INSTDIR\${{MAINBINARYNAME}}.exe$\" ${{ARGUMENTS}} $\"${{ARGUMENT}}$\""
!macroend

!macro LotpixSubmenu KEY LABEL
  WriteRegStr SHCTX "${{KEY}}" "MUIVerb" "${{LABEL}}"
  WriteRegStr SHCTX "${{KEY}}" "Icon" "$\"$INSTDIR\${{MAINBINARYNAME}}.exe$\",0"
  WriteRegStr SHCTX "${{KEY}}" "MultiSelectModel" "Player"
  WriteRegStr SHCTX "${{KEY}}" "SubCommands" ""
!macroend

!macro LotpixMenuRemove CLASS _ARGUMENT
  DeleteRegKey SHCTX "Software\Classes\${{CLASS}}\shell\Lotpix.Compress"
  DeleteRegKey SHCTX "Software\Classes\${{CLASS}}\shell\Lotpix.Resize"
  DeleteRegKey SHCTX "Software\Classes\${{CLASS}}\shell\Lotpix.Open"
!macroend

; Libellés dans la langue de l'installation
{lang_strings}

!macro LotpixMenu CLASS ARGUMENT
  ; Repart de zéro : une version précédente a pu laisser d'autres entrées
  !insertmacro LotpixMenuRemove "${{CLASS}}" "${{ARGUMENT}}"

{menu}
  !insertmacro LotpixCommand "Software\Classes\${{CLASS}}\shell\Lotpix.Open" "$(lotpix_open)" "open" "${{ARGUMENT}}"
  WriteRegStr SHCTX "Software\Classes\${{CLASS}}\shell\Lotpix.Open" "Icon" "$\"$INSTDIR\${{MAINBINARYNAME}}.exe$\",0"
!macroend

!macro LotpixMenus MACRO
  !insertmacro ${{MACRO}} "SystemFileAssociations\.jpg" "%1"
  !insertmacro ${{MACRO}} "SystemFileAssociations\.jpeg" "%1"
  !insertmacro ${{MACRO}} "SystemFileAssociations\.png" "%1"
  !insertmacro ${{MACRO}} "SystemFileAssociations\.gif" "%1"
  !insertmacro ${{MACRO}} "SystemFileAssociations\.tif" "%1"
  !insertmacro ${{MACRO}} "SystemFileAssociations\.tiff" "%1"
  !insertmacro ${{MACRO}} "SystemFileAssociations\.webp" "%1"
  !insertmacro ${{MACRO}} "Directory" "%1"
  !insertmacro ${{MACRO}} "Directory\Background" "%V"
!macroend

Section ContextMenu
  !insertmacro LotpixMenus LotpixMenu
SectionEnd

Section un.ContextMenu
  !insertmacro LotpixMenus LotpixMenuRemove
SectionEnd
'''
{NSIS_END}"#,
        lang_strings = lang_strings.join("\n"),
        menu = menu.join("\n"),
    )
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Fichiers attendus et leur contenu.
fn expected_files() -> Vec<(PathBuf, String)> {
    let dolphin = root().join("packaging/linux/dolphin");
    let nemo = root().join("packaging/linux/nemo");
    let chooser = "sh /usr/share/nemo/actions/lotpix-menu.sh";
    vec![
        (
            dolphin.join("lotpix-1-compress.desktop"),
            dolphin_submenu(Submenu::Compress),
        ),
        (
            dolphin.join("lotpix-2-resize.desktop"),
            dolphin_submenu(Submenu::Resize),
        ),
        (dolphin.join("lotpix-3-open.desktop"), dolphin_open()),
        (
            nemo.join("lotpix-1-compress.nemo_action"),
            nemo_action(
                || format!("{}…", t!("menu-compress")),
                || t!("menu-compress-comment"),
                &format!("{chooser} compress %F"),
                true,
            ),
        ),
        (
            nemo.join("lotpix-2-resize.nemo_action"),
            nemo_action(
                || format!("{}…", t!("menu-resize")),
                || t!("menu-resize-comment"),
                &format!("{chooser} resize %F"),
                true,
            ),
        ),
        (
            nemo.join("lotpix-3-open.nemo_action"),
            nemo_action(
                || t!("menu-open"),
                || t!("menu-open-comment"),
                "lotpix open %F",
                false,
            ),
        ),
    ]
}

fn read(path: &Path) -> String {
    // Une copie de travail Windows peut avoir des fins de ligne CRLF
    fs::read_to_string(path)
        .unwrap_or_default()
        .replace("\r\n", "\n")
}

#[test]
fn context_menu_files_are_up_to_date() {
    let update = std::env::var_os("LOTPIX_UPDATE_MENUS").is_some();
    let mut stale = Vec::new();

    for (path, expected) in expected_files() {
        if read(&path) != expected {
            if update {
                fs::write(&path, &expected).unwrap();
            } else {
                stale.push(path.display().to_string());
            }
        }
    }

    let manifest_path = root().join("Cargo.toml");
    let manifest = read(&manifest_path);
    let start = manifest
        .find(NSIS_START)
        .expect("début du bloc NSIS dans Cargo.toml");
    let end = manifest[start..]
        .find(NSIS_END)
        .map(|end| start + end + NSIS_END.len())
        .expect("fin du bloc NSIS dans Cargo.toml");
    let expected = nsis_section();
    if manifest[start..end] != expected {
        if update {
            let updated = format!("{}{expected}{}", &manifest[..start], &manifest[end..]);
            fs::write(&manifest_path, updated).unwrap();
        } else {
            stale.push(manifest_path.display().to_string());
        }
    }

    assert!(
        stale.is_empty(),
        "menus contextuels à régénérer avec `LOTPIX_UPDATE_MENUS=1 cargo test` : {stale:?}"
    );
}
