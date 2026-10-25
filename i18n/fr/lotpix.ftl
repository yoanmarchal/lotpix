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
