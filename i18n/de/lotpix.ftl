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
