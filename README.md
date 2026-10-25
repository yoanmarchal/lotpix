# Lotpix

**Bulk image optimizer** : une application de bureau pour convertir, compresser et redimensionner des images par lot, écrite en Rust avec [Iced](https://iced.rs).

[![Build and Release](https://github.com/yoanmarchal/lotpix/actions/workflows/release.yml/badge.svg)](https://github.com/yoanmarchal/lotpix/actions/workflows/release.yml)

> **Vous utilisiez Image Converter (version 1, Electron) ?** Lotpix en est la réécriture complète, sous un nouveau nom : il ne se met pas à jour depuis la version 1. Désinstallez l'ancienne version, puis installez la nouvelle depuis les [releases GitHub](https://github.com/yoanmarchal/lotpix/releases). L'historique des conversions n'est pas repris.

## Fonctionnalités

- 🖼️ **Conversion d'images multi-formats**
  - Formats de sortie : WebP (meilleur rapport qualité/taille), JPEG (compatible partout), PNG (sans perte, avec transparence), AVIF (format moderne haute performance)
  - « Format d'origine » pour seulement redimensionner ou recompresser (les GIF et TIFF sont alors enregistrés en PNG)
  - Formats d'entrée : JPEG, PNG, GIF, TIFF, WebP
  - Orientation EXIF appliquée : les photos prises en portrait restent en portrait
  - GIF animés conservés en WebP animé
  - Transparence remplacée par un fond blanc en JPEG

- 📐 **Redimensionnement par lot** (optionnel)
  - Par largeur, hauteur, côté le plus long ou pourcentage, toujours à proportions conservées
  - Option « Ne pas agrandir les images plus petites », cochée par défaut
  - Dimensions finales de chaque image affichées avant la conversion

- ⚙️ **Options de conversion**
  - Contrôle de la qualité (1 à 100 %, sauf PNG qui est sans perte)
  - Fichiers convertis à côté des originaux, ou dans un dossier de sortie au choix
  - Aucun fichier écrasé : si `photo.webp` existe déjà, la sortie devient `photo-1.webp`

- 🎯 **Interface**
  - Glisser-déposer des images, ou ouverture depuis l'Explorateur (« Ouvrir avec »)
  - Miniatures, suivi de chaque conversion, gain de poids
  - Mode sombre/clair

- 🖱️ **Menu contextuel** (Explorateur Windows, Dolphin, Nemo)
  - Sur des images ou des dossiers (sous-dossiers compris), sans ouvrir de fenêtre, avec la qualité recommandée pour chaque format (WebP 80, JPEG 82, AVIF 65, PNG sans perte). Les fichiers sont écrits à côté des originaux, et une notification résume le gain
  - « Compresser avec Lotpix » : WebP, JPEG, PNG, AVIF ou format d'origine
  - « Redimensionner avec Lotpix » : tailles standard (4K, Full HD, HD, Web, Miniature, 50 %), en gardant le format d'origine, ou formats des réseaux sociaux (Instagram carré et portrait, Story/Reels/TikTok, Facebook/LinkedIn, X, miniature YouTube, Pinterest), redimensionnés puis recadrés au centre à la taille exacte et écrits en JPEG. Une image plus petite que le cadre est recadrée sans être agrandie
  - « Ouvrir dans Lotpix » pour choisir d'autres réglages avant de convertir
  - Sous Windows 11, les entrées sont dans « Afficher plus d'options » (ou Maj + clic droit) ; Nemo n'ayant pas de sous-menus, ses entrées « Compresser… » et « Redimensionner… » font choisir le format ou la taille dans une liste (elles demandent zenity, installé par défaut sur Cinnamon)
  - En ligne de commande : `lotpix compress --format <format>`, `lotpix resize --size <taille>`, ou `lotpix compress` seul pour les réglages de l'interface (`lotpix --help` liste les tailles)

- 📊 **Historique des conversions**
  - Les 100 dernières conversions, avec poids et dimensions avant et après
  - Accès rapide aux fichiers convertis

- 🔔 **Nouvelles versions signalées**
  - Au démarrage, l'application interroge les releases GitHub en tâche de fond et affiche un bandeau si une version plus récente existe
  - Le bouton « Télécharger » ouvre la page de la release : l'installation reste manuelle
  - Désactivable par la case « Vérifier les mises à jour » en bas de la fenêtre ; c'est le seul accès réseau de l'application

## Technologies utilisées

- Rust
- [Iced](https://iced.rs) pour l'interface, sans navigateur embarqué
- [`image`](https://crates.io/crates/image), [`webp`](https://crates.io/crates/webp), [`jpeg-encoder`](https://crates.io/crates/jpeg-encoder) et [`ravif`](https://crates.io/crates/ravif) pour les codecs
- [`fast_image_resize`](https://crates.io/crates/fast_image_resize) pour le redimensionnement
- [`cargo-packager`](https://crates.io/crates/cargo-packager) pour les installeurs

## Développement

Prérequis : [Rust](https://rustup.rs) stable (édition 2024, donc Rust 1.85 ou plus récent). Sous Windows, les outils de build C++ de Visual Studio sont nécessaires pour compiler libwebp.

```bash
git clone https://github.com/yoanmarchal/lotpix.git
cd lotpix
cargo run --release
```

## Commandes disponibles

- `cargo run --release` - Lance l'application
- `cargo test` - Lance les tests ; le rendu de chaque écran est aussi écrit dans `target/captures/`
- `cargo fmt` et `cargo clippy --all-targets` - Formate et vérifie le code
- `cargo packager --release --formats nsis` - Construit l'installeur Windows dans `release/`
- `cargo packager --release --formats deb,appimage` - Construit les paquets Linux dans `release/`
- `cargo run --release --example compare -- <dossier de sortie> <images…>` - Mesure le temps et le poids de sortie de chaque format

Les deux commandes de packaging demandent `cargo install cargo-packager --locked`.

## Organisation du code

- `src/conversion.rs` - Conversion et redimensionnement, sans dépendance à l'interface
- `src/history.rs`, `src/preferences.rs`, `src/storage.rs` - Données enregistrées dans le dossier de l'utilisateur
- `src/format.rs` - Mise en forme française des tailles, pourcentages et dates
- `src/update.rs` - Recherche d'une version plus récente dans les releases GitHub
- `src/ui/` - Interface Iced : état et messages (`mod.rs`), thème (`theme.rs`), écrans (`convert.rs`, `history.rs`)
- `tests/conversion.rs` - Tests de conversion sur de vraies images

## Publier une version

Modifiez `version` dans `Cargo.toml` et fusionnez sur `main` : le workflow crée le tag `vX.Y.Z`, la release GitHub et y dépose les installeurs Windows et Linux.

Sous Windows, le workflow attend deux validations dans SignPath (l'application, puis l'installeur), pendant une heure au plus chacune. Il faut pour cela le secret `SIGNPATH_API_TOKEN` et les variables `SIGNPATH_ORGANIZATION_ID`, `SIGNPATH_PROJECT_SLUG` et `SIGNPATH_SIGNING_POLICY_SLUG` dans les réglages du dépôt ; sans le secret, l'installeur est publié sans signature. Les configurations d'artefacts à créer dans SignPath sont dans [`.signpath/artifact-configurations/`](.signpath/artifact-configurations/), sous les noms `executable` et `installer`.

## Configuration système requise

### Windows
- Windows 10 ou plus récent
- Architecture x64

### Linux
- Ubuntu 22.04 ou plus récent, ou distribution équivalente
- Architecture x64 ou arm64

## Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org).

Les exécutables Windows (`lotpix.exe` et l'installeur) sont compilés et signés par le workflow GitHub Actions [`release.yml`](.github/workflows/release.yml), à partir du code de ce dépôt. Chaque signature est validée à la main avant d'être appliquée.

Équipe :
- Auteurs et relecteurs : [Yoan Marchal](https://github.com/yoanmarchal)
- Approbateurs des signatures : [Yoan Marchal](https://github.com/yoanmarchal)

### Confidentialité

Lotpix traite les images sur l'ordinateur : aucune image, aucun nom de fichier ni aucune donnée d'utilisation n'est envoyé. Le seul accès réseau est la vérification des nouvelles versions au démarrage, qui interroge l'API des releases GitHub ; elle se désactive par la case « Vérifier les mises à jour » en bas de la fenêtre. Cette requête est soumise à la [politique de confidentialité de GitHub](https://docs.github.com/site-policy/privacy-policies/github-general-privacy-statement).

## License

[MIT](LICENSE) © Yoan Marchal. La police [Inter](https://rsms.me/inter/) est distribuée sous licence SIL Open Font License (`assets/fonts/Inter-LICENSE.txt`).
