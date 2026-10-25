# Deutsch. Maschinelle Übersetzung: muss noch von einer Muttersprachlerin oder einem Muttersprachler geprüft werden.

## Sprache

language = Sprache
language-system = System

## Kopfzeile

tab-convert = Konvertieren
tab-history = Verlauf
theme-to-light = Zum hellen Modus wechseln
theme-to-dark = Zum dunklen Modus wechseln

## Update-Hinweis

update-available = Version { $version } ist verfügbar
update-download = Herunterladen
update-dismiss = Schließen

## Statusleiste

status-images =
    { $count ->
        [one] { $count } Bild
       *[other] { $count } Bilder
    }
status-converted = { $count } konvertiert
status-failed = { $count } fehlgeschlagen
status-converting = Konvertierung läuft… { $done } / { $total }
status-check-updates = Nach Updates suchen

## Bildschirm Konvertieren: Ablagebereich

images-loading = Bilder werden geladen…
drop-hovered = Bilder hier ablegen
drop-title = Bilder hierher ziehen und ablegen
drop-subtitle = oder klicken, um Dateien auszuwählen (JPEG, PNG, GIF, TIFF, WebP)
drop-choose = Bilder auswählen

## Bildschirm Konvertieren: Bilderliste

list-empty = Keine Bilder hinzugefügt
list-title = Bilder ({ $count })
list-remove-all = Alle entfernen
image-converting = Wird konvertiert
image-converted = Konvertiert
image-failed = Fehler
image-remove = Aus der Liste entfernen
image-gain = Ersparnis: { $gain }
open = Öffnen

## Bildschirm Konvertieren: Einstellungen

settings-title = Konvertierungseinstellungen
format-label = Ausgabeformat
format-webp = WebP - Klein und hochwertig
format-jpg = JPEG - Überall kompatibel
format-png = PNG - Verlustfrei mit Transparenz
format-avif = AVIF - Modernes Hochleistungsformat
format-original = Originalformat - Nur zum Skalieren
quality-label = Qualität: { $quality } %
quality-low = Niedrig
quality-high = Hoch
quality-hint-lossless = PNG ist verlustfrei: Die Qualität wird nicht angewendet
quality-hint-low = Niedrige Qualität, kleinere Datei
quality-hint-balanced = Ausgewogene Qualität
quality-hint-high = Hohe Qualität, größere Datei
resize-label = Skalieren
resize-longest-side = Längste Seite
resize-width = Breite
resize-height = Höhe
resize-percent = Prozent
resize-placeholder = Größe
resize-keeps-ratio = Die Seitenverhältnisse des Bildes bleiben erhalten
resize-no-upscale = Kleinere Bilder nicht vergrößern
output-label = Ausgabeordner (optional)
output-same-folder = Gleicher Ordner wie das Original
output-change = Ausgabeordner ändern
output-choose = Ausgabeordner auswählen
output-reset = Zurück zum Originalordner
convert-button = Bilder konvertieren
convert-running = Konvertierung läuft…
convert-hint-empty = Bitte fügen Sie Bilder zum Konvertieren hinzu
convert-hint-done = Alle Bilder sind verarbeitet: Ändern Sie eine Einstellung, um sie erneut zu konvertieren

## Bildschirm Verlauf

history-empty = Noch keine Konvertierungen
history-title = Konvertierungsverlauf
history-clear = Verlauf leeren
history-clear-question = Gesamten Verlauf leeren?
history-clear-cancel = Abbrechen
history-clear-confirm = Leeren
history-column-file = DATEI
history-column-original = ORIGINAL
history-column-converted = KONVERTIERT
history-column-dimensions = ABMESSUNGEN
history-column-gain = ERSPARNIS
history-column-date = DATUM
history-column-actions = AKTIONEN

## Meldungen der Statusleiste

notice-preferences-not-saved = Die Einstellungen konnten nicht gespeichert werden
notice-settings-not-saved = Die Konvertierungseinstellungen konnten nicht gespeichert werden
notice-history-not-saved = Der Verlauf konnte nicht gespeichert werden
notice-download-page = Die Download-Seite konnte nicht geöffnet werden
notice-ignored-files =
    { $count ->
        [one] { $count } Datei ignoriert
       *[other] { $count } Dateien ignoriert
    }: Format nicht unterstützt oder Datei unlesbar
notice-file-missing = Die Datei existiert nicht mehr: { $name }
notice-cannot-open = { $name } konnte nicht geöffnet werden
conversion-unexpected-error = Unerwarteter Fehler bei der Konvertierung
file-unknown = unbekannt

## Dateiauswahl

dialog-choose-images = Bilder auswählen
dialog-images-filter = Bilder
dialog-choose-output = Ausgabeordner auswählen

## Konvertierungsfehler

error-invalid-path = Der Pfad der Quelldatei ist ungültig oder fehlt
error-unsupported-extension = Nicht unterstützte Dateiendung: { $extension }
error-no-extension = (keine)
error-source-not-found = Die Quelldatei existiert nicht: { $path }
error-not-a-file = Der Pfad ist keine Datei: { $path }
error-invalid-quality = Die Qualität muss eine Zahl zwischen 1 und 100 sein
error-invalid-resize = Die gewünschte Größe muss zwischen 1 und { $max } { $unit } liegen
error-too-large = Das Bild wäre { $width } × { $height } px groß: Das Format { $format } ist auf { $max } px pro Seite begrenzt
error-too-many-pixels = Das Bild wäre { $width } × { $height } px groß: Das ist zu viel für die Verarbeitung (maximal { $max } Megapixel)
error-empty-output = Die Ausgabedatei ist leer
error-decode = Bild unlesbar: { $error }
error-resize = Skalierung fehlgeschlagen: { $error }
error-encode = Kodierung fehlgeschlagen: { $error }
error-resize-buffer = ungültiger Ausgabepuffer
error-webp-config = ungültige WebP-Konfiguration
error-webp-version = inkompatible libwebp-Version
error-webp-animation = Animations-Encoder konnte nicht erstellt werden
error-webp-dimensions = Einzelbild mit unerwarteten Abmessungen
error-webp-unknown = unbekannter Fehler

## Befehlszeile und Benachrichtigungen

cli-usage = Verwendung:
cli-files = Dateien…
cli-files-or-folders = Dateien oder Ordner…
cli-format = Format
cli-size = Größe
cli-help-open = Öffnet die Oberfläche mit diesen Bildern
cli-help-open-gather = Ebenso, fasst gleichzeitige Starts zusammen
cli-help-compress-format = Komprimiert mit der empfohlenen Qualität
cli-help-resize = Skaliert mit der empfohlenen Qualität
cli-help-compress-saved = Komprimiert mit den Einstellungen der Oberfläche
cli-help-folders = Ordner werden mit ihren Unterordnern verarbeitet. Mit --format und --size werden die Bilder neben den Originalen gespeichert.
cli-help-formats = Formate:
cli-help-original-format = Originalformat
cli-help-sizes = Größen (Formate sozialer Netzwerke werden mittig zugeschnitten und als JPEG gespeichert):
cli-unknown-format = Unbekanntes Format: { $value }
cli-unknown-size = Unbekannte Größe: { $value }
cli-missing-size = Größe fehlt: --size <Größe>
report-compressed =
    { $count ->
        [one] { $count } Bild komprimiert
       *[other] { $count } Bilder komprimiert
    }
report-resized =
    { $count ->
        [one] { $count } Bild skaliert
       *[other] { $count } Bilder skaliert
    }
report-failures =
    { $count ->
        [one] , { $count } Fehler
       *[other] , { $count } Fehler
    }
report-failure = { $name }: { $error }
report-impossible = Komprimierung nicht möglich
report-invalid-saved-resize = Die gespeicherte Skalierungsgröße ist ungültig: Korrigieren Sie sie in Lotpix
report-no-images = Keine Bilder zu verarbeiten
report-supported-formats = Unterstützte Formate: JPEG, PNG, GIF, TIFF und WebP

## Größen des Kontextmenüs

size-4k = 4K (3840 px)
size-full-hd = Full HD (1920 px)
size-hd = HD (1280 px)
size-web = Web (1024 px)
size-thumbnail = Miniatur (640 px)
size-half = 50 %
size-instagram-square = Instagram quadratisch (1080 × 1080)
size-instagram-portrait = Instagram Hochformat (1080 × 1350)
size-story = Story, Reels, TikTok (1080 × 1920)
size-facebook = Facebook, LinkedIn (1200 × 630)
size-x = X (1600 × 900)
size-youtube = YouTube-Vorschaubild (1280 × 720)
size-pinterest = Pinterest (1000 × 1500)

## Kontextmenüs der Dateimanager (Windows Explorer, Dolphin, Nemo)

menu-compress = Mit Lotpix komprimieren
menu-resize = Mit Lotpix skalieren
menu-open = In Lotpix öffnen
menu-format-webp = WebP
menu-format-jpg = JPEG
menu-format-png = PNG (verlustfrei)
menu-format-avif = AVIF
menu-format-original = Originalformat
menu-compress-comment = Komprimiert mit der empfohlenen Qualität im gewählten Format
menu-resize-comment = Skaliert auf eine Standardgröße oder das Format eines sozialen Netzwerks
menu-open-comment = Öffnet die Bilder in Lotpix, um die Einstellungen zu wählen
menu-choose-format = Format, mit der empfohlenen Qualität:
menu-choose-size = Größe (Formate sozialer Netzwerke werden zugeschnitten und als JPEG gespeichert):
