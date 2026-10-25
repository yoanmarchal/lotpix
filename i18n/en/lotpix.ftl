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

## Conversion errors

error-invalid-path = The source file path is invalid or missing
error-unsupported-extension = Unsupported extension: { $extension }
error-no-extension = (none)
error-source-not-found = The source file does not exist: { $path }
error-not-a-file = The path is not a file: { $path }
error-invalid-quality = Quality must be a number between 1 and 100
error-invalid-resize = The requested size must be between 1 and { $max } { $unit }
error-too-large = The image would be { $width } × { $height } px: the { $format } format is limited to { $max } px per side
error-too-many-pixels = The image would be { $width } × { $height } px: that is too much to process (maximum { $max } megapixels)
error-empty-output = The output file is empty
error-decode = Unreadable image: { $error }
error-resize = Resizing failed: { $error }
error-encode = Encoding failed: { $error }
error-resize-buffer = invalid output buffer
error-webp-config = invalid WebP configuration
error-webp-version = incompatible libwebp version
error-webp-animation = could not create the animation encoder
error-webp-dimensions = frame with unexpected dimensions
error-webp-unknown = unknown error

## Command line and notifications

cli-usage = Usage:
cli-files = files…
cli-files-or-folders = files or folders…
cli-format = format
cli-size = size
cli-help-open = Opens the interface with these images
cli-help-open-gather = Same, grouping simultaneous launches
cli-help-compress-format = Compresses with the recommended quality
cli-help-resize = Resizes with the recommended quality
cli-help-compress-saved = Compresses with the interface settings
cli-help-folders = Folders are processed with their subfolders. With --format and --size, the images are written next to the originals.
cli-help-formats = Formats:
cli-help-original-format = original format
cli-help-sizes = Sizes (social network formats are cropped to the center and saved as JPEG):
cli-unknown-format = Unknown format: { $value }
cli-unknown-size = Unknown size: { $value }
cli-missing-size = Missing size: --size <size>
report-compressed =
    { $count ->
        [one] { $count } image compressed
       *[other] { $count } images compressed
    }
report-resized =
    { $count ->
        [one] { $count } image resized
       *[other] { $count } images resized
    }
report-failures =
    { $count ->
        [one] , { $count } failure
       *[other] , { $count } failures
    }
report-failure = { $name }: { $error }
report-impossible = Cannot compress
report-invalid-saved-resize = The saved resize value is invalid: correct it in Lotpix
report-no-images = No images to process
report-supported-formats = Supported formats: JPEG, PNG, GIF, TIFF and WebP

## Sizes of the context menu

size-4k = 4K (3840 px)
size-full-hd = Full HD (1920 px)
size-hd = HD (1280 px)
size-web = Web (1024 px)
size-thumbnail = Thumbnail (640 px)
size-half = 50%
size-instagram-square = Instagram square (1080 × 1080)
size-instagram-portrait = Instagram portrait (1080 × 1350)
size-story = Story, Reels, TikTok (1080 × 1920)
size-facebook = Facebook, LinkedIn (1200 × 630)
size-x = X (1600 × 900)
size-youtube = YouTube thumbnail (1280 × 720)
size-pinterest = Pinterest (1000 × 1500)
