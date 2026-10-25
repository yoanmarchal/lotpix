# English: reference language. Every key used in the code must exist here.

## Language

language = Language
language-system = System

## Header

tab-convert = Convert
tab-history = History
theme-to-light = Switch to light mode
theme-to-dark = Switch to dark mode

## Update banner

update-available = Version { $version } is available
update-download = Download
update-dismiss = Close

## Status bar

status-images =
    { $count ->
        [one] { $count } image
       *[other] { $count } images
    }
status-converted = { $count } converted
status-failed = { $count } failed
status-converting = Converting… { $done } / { $total }
status-check-updates = Check for updates

## Convert screen: drop zone

images-loading = Loading images…
drop-hovered = Drop the images here
drop-title = Drag and drop your images here
drop-subtitle = or click to choose files (JPEG, PNG, GIF, TIFF, WebP)
drop-choose = Choose images

## Convert screen: image list

list-empty = No images added
list-title = Images ({ $count })
list-remove-all = Remove all
image-converting = Converting
image-converted = Converted
image-failed = Error
image-remove = Remove from the list
image-gain = Saved: { $gain }
open = Open

## Convert screen: settings

settings-title = Conversion settings
format-label = Output format
format-webp = WebP - Best quality/size ratio
format-jpg = JPEG - Compatible everywhere
format-png = PNG - Lossless with transparency
format-avif = AVIF - Modern high-performance format
format-original = Original format - To resize only
quality-label = Quality: { $quality }%
quality-low = Low
quality-high = High
quality-hint-lossless = PNG is lossless: quality does not apply
quality-hint-low = Low quality, smaller file
quality-hint-balanced = Balanced quality
quality-hint-high = High quality, larger file
resize-label = Resize
resize-longest-side = Longest side
resize-width = Width
resize-height = Height
resize-percent = Percentage
resize-placeholder = Size
resize-keeps-ratio = The image proportions are kept
resize-no-upscale = Do not enlarge smaller images
output-label = Output folder (optional)
output-same-folder = Same folder as the original
output-change = Change the output folder
output-choose = Choose an output folder
output-reset = Back to the original folder
convert-button = Convert images
convert-running = Converting…
convert-hint-empty = Please add images to convert
convert-hint-done = All images are processed: change a setting to convert them again

## History screen

history-empty = No conversions yet
history-title = Conversion history
history-clear = Clear history
history-clear-question = Clear the whole history?
history-clear-cancel = Cancel
history-clear-confirm = Clear
history-column-file = FILE
history-column-original = ORIGINAL
history-column-converted = CONVERTED
history-column-dimensions = DIMENSIONS
history-column-gain = SAVED
history-column-date = DATE
history-column-actions = ACTIONS

## Status bar messages

notice-preferences-not-saved = Could not save the preferences
notice-settings-not-saved = Could not save the settings
notice-history-not-saved = Could not save the history
notice-download-page = Could not open the download page
notice-ignored-files =
    { $count ->
        [one] { $count } file ignored
       *[other] { $count } files ignored
    }: unsupported format or unreadable file
notice-file-missing = The file no longer exists: { $name }
notice-cannot-open = Could not open { $name }
conversion-unexpected-error = Unexpected error during the conversion
file-unknown = unknown

## File dialogs

dialog-choose-images = Choose images
dialog-images-filter = Images
dialog-choose-output = Choose the output folder
