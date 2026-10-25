# Français

## Langue

language = Langue
language-system = Système

## En-tête

tab-convert = Convertir
tab-history = Historique
theme-to-light = Passer en mode clair
theme-to-dark = Passer en mode sombre

## Bandeau de mise à jour

update-available = La version { $version } est disponible
update-download = Télécharger
update-dismiss = Fermer

## Barre de statut

status-images =
    { $count ->
        [one] { $count } image
       *[other] { $count } images
    }
status-converted =
    { $count ->
        [one] { $count } convertie
       *[other] { $count } converties
    }
status-failed = { $count } en erreur
status-converting = Conversion en cours… { $done } / { $total }
status-check-updates = Vérifier les mises à jour

## Écran Convertir : zone de dépôt

images-loading = Chargement des images…
drop-hovered = Déposez les images ici
drop-title = Glissez-déposez vos images ici
drop-subtitle = ou cliquez pour choisir des fichiers (JPEG, PNG, GIF, TIFF, WebP)
drop-choose = Choisir des images

## Écran Convertir : liste des images

list-empty = Aucune image ajoutée
list-title = Images ({ $count })
list-remove-all = Tout retirer
image-converting = Conversion en cours
image-converted = Convertie
image-failed = Erreur
image-remove = Retirer de la liste
image-gain = Gain : { $gain }
open = Ouvrir

## Écran Convertir : réglages

settings-title = Paramètres de conversion
format-label = Format de sortie
format-webp = WebP - Meilleur rapport qualité/taille
format-jpg = JPEG - Compatible partout
format-png = PNG - Sans perte avec transparence
format-avif = AVIF - Format moderne haute performance
format-original = Format d'origine - Pour seulement redimensionner
quality-label = Qualité : { $quality } %
quality-low = Basse
quality-high = Haute
quality-hint-lossless = Le PNG est sans perte : la qualité ne s'applique pas
quality-hint-low = Qualité basse, fichier plus petit
quality-hint-balanced = Qualité équilibrée
quality-hint-high = Haute qualité, fichier plus grand
resize-label = Redimensionner
resize-longest-side = Côté le plus long
resize-width = Largeur
resize-height = Hauteur
resize-percent = Pourcentage
resize-placeholder = Taille
resize-keeps-ratio = Les proportions de l'image sont conservées
resize-no-upscale = Ne pas agrandir les images plus petites
output-label = Dossier de sortie (optionnel)
output-same-folder = Même dossier que l'original
output-change = Changer le dossier de sortie
output-choose = Choisir un dossier de sortie
output-reset = Revenir au dossier d'origine
convert-button = Convertir les images
convert-running = Conversion en cours…
convert-hint-empty = Veuillez ajouter des images à convertir
convert-hint-done = Toutes les images sont traitées : changez un réglage pour les reconvertir

## Écran Historique

history-empty = Aucune conversion pour le moment
history-title = Historique des conversions
history-clear = Vider l'historique
history-clear-question = Vider tout l'historique ?
history-clear-cancel = Annuler
history-clear-confirm = Vider
history-column-file = FICHIER
history-column-original = ORIGINE
history-column-converted = CONVERTIE
history-column-dimensions = DIMENSIONS
history-column-gain = GAIN
history-column-date = DATE
history-column-actions = ACTIONS

## Messages de la barre de statut

notice-preferences-not-saved = Impossible d'enregistrer les préférences
notice-settings-not-saved = Impossible d'enregistrer les réglages
notice-history-not-saved = Impossible d'enregistrer l'historique
notice-download-page = Impossible d'ouvrir la page de téléchargement
notice-ignored-files =
    { $count ->
        [one] { $count } fichier ignoré
       *[other] { $count } fichiers ignorés
    } : format non pris en charge ou fichier illisible
notice-file-missing = Le fichier n'existe plus : { $name }
notice-cannot-open = Impossible d'ouvrir { $name }
conversion-unexpected-error = Erreur inattendue pendant la conversion
file-unknown = inconnu

## Sélecteurs de fichiers

dialog-choose-images = Choisir des images
dialog-images-filter = Images
dialog-choose-output = Choisir le dossier de sortie

## Erreurs de conversion

error-invalid-path = Le chemin du fichier source est invalide ou manquant
error-unsupported-extension = Extension non supportée : { $extension }
error-no-extension = (aucune)
error-source-not-found = Le fichier source n'existe pas : { $path }
error-not-a-file = Le chemin ne désigne pas un fichier : { $path }
error-invalid-quality = La qualité doit être un nombre entre 1 et 100
error-invalid-resize = La taille demandée doit être comprise entre 1 et { $max } { $unit }
error-too-large = L'image ferait { $width } × { $height } px : le format { $format } est limité à { $max } px de côté
error-too-many-pixels = L'image ferait { $width } × { $height } px : c'est trop pour être traité (maximum { $max } mégapixels)
error-empty-output = Le fichier de sortie est vide
error-decode = Image illisible : { $error }
error-resize = Échec du redimensionnement : { $error }
error-encode = Échec de l'encodage : { $error }
error-resize-buffer = tampon de sortie invalide
error-webp-config = configuration WebP invalide
error-webp-version = version de libwebp incompatible
error-webp-animation = impossible de créer l'encodeur d'animation
error-webp-dimensions = image aux dimensions inattendues
error-webp-unknown = erreur inconnue

## Ligne de commande et notifications

cli-usage = Utilisation :
cli-files = fichiers…
cli-files-or-folders = fichiers ou dossiers…
cli-format = format
cli-size = taille
cli-help-open = Ouvre l'interface avec ces images
cli-help-open-gather = Idem, en regroupant les lancements simultanés
cli-help-compress-format = Compresse avec la qualité recommandée
cli-help-resize = Redimensionne avec la qualité recommandée
cli-help-compress-saved = Compresse avec les réglages de l'interface
cli-help-folders = Les dossiers sont parcourus avec leurs sous-dossiers. Avec --format et --size, les images sont écrites à côté des originales.
cli-help-formats = Formats :
cli-help-original-format = format d'origine
cli-help-sizes = Tailles (les formats des réseaux sociaux sont recadrés au centre et écrits en JPEG) :
cli-unknown-format = Format inconnu : { $value }
cli-unknown-size = Taille inconnue : { $value }
cli-missing-size = Taille manquante : --size <taille>
report-compressed =
    { $count ->
        [one] { $count } image compressée
       *[other] { $count } images compressées
    }
report-resized =
    { $count ->
        [one] { $count } image redimensionnée
       *[other] { $count } images redimensionnées
    }
report-failures =
    { $count ->
        [one] , { $count } échec
       *[other] , { $count } échecs
    }
report-failure = { $name } : { $error }
report-impossible = Compression impossible
report-invalid-saved-resize = La taille de redimensionnement enregistrée est invalide : corrigez-la dans Lotpix
report-no-images = Aucune image à traiter
report-supported-formats = Formats pris en charge : JPEG, PNG, GIF, TIFF et WebP

## Tailles du menu contextuel

size-4k = 4K (3840 px)
size-full-hd = Full HD (1920 px)
size-hd = HD (1280 px)
size-web = Web (1024 px)
size-thumbnail = Miniature (640 px)
size-half = 50 %
size-instagram-square = Instagram carré (1080 × 1080)
size-instagram-portrait = Instagram portrait (1080 × 1350)
size-story = Story, Reels, TikTok (1080 × 1920)
size-facebook = Facebook, LinkedIn (1200 × 630)
size-x = X (1600 × 900)
size-youtube = Miniature YouTube (1280 × 720)
size-pinterest = Pinterest (1000 × 1500)

## Menus contextuels des gestionnaires de fichiers (Explorateur Windows, Dolphin, Nemo)

menu-compress = Compresser avec Lotpix
menu-resize = Redimensionner avec Lotpix
menu-open = Ouvrir dans Lotpix
menu-format-webp = WebP
menu-format-jpg = JPEG
menu-format-png = PNG (sans perte)
menu-format-avif = AVIF
menu-format-original = Format d'origine
menu-compress-comment = Compresse avec la qualité recommandée, dans le format choisi
menu-resize-comment = Redimensionne à une taille standard ou au format d'un réseau social
menu-open-comment = Ouvre les images dans Lotpix pour choisir les réglages
menu-choose-format = Format, avec la qualité recommandée :
menu-choose-size = Taille (les formats des réseaux sociaux sont recadrés et écrits en JPEG) :
