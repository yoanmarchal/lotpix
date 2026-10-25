# Español. Traducción automática: pendiente de revisión por un hablante nativo.

## Idioma

language = Idioma
language-system = Sistema

## Encabezado

tab-convert = Convertir
tab-history = Historial
theme-to-light = Cambiar al modo claro
theme-to-dark = Cambiar al modo oscuro

## Aviso de actualización

update-available = La versión { $version } está disponible
update-download = Descargar
update-dismiss = Cerrar

## Barra de estado

status-images =
    { $count ->
        [one] { $count } imagen
       *[other] { $count } imágenes
    }
status-converted =
    { $count ->
        [one] { $count } convertida
       *[other] { $count } convertidas
    }
status-failed = { $count } con error
status-converting = Convirtiendo… { $done } / { $total }
status-check-updates = Buscar actualizaciones

## Pantalla Convertir: zona de carga

images-loading = Cargando imágenes…
drop-hovered = Suelte las imágenes aquí
drop-title = Arrastre y suelte sus imágenes aquí
drop-subtitle = o haga clic para elegir archivos (JPEG, PNG, GIF, TIFF, WebP)
drop-choose = Elegir imágenes

## Pantalla Convertir: lista de imágenes

list-empty = No se ha añadido ninguna imagen
list-title = Imágenes ({ $count })
list-remove-all = Quitar todas
image-converting = Convirtiendo
image-converted = Convertida
image-failed = Error
image-remove = Quitar de la lista
image-gain = Ahorro: { $gain }
open = Abrir

## Pantalla Convertir: ajustes

settings-title = Ajustes de conversión
format-label = Formato de salida
format-webp = WebP - Mejor relación calidad/tamaño
format-jpg = JPEG - Compatible en todas partes
format-png = PNG - Sin pérdida y con transparencia
format-avif = AVIF - Formato moderno y eficiente
format-original = Formato original - Solo redimensionar
quality-label = Calidad: { $quality } %
quality-low = Baja
quality-high = Alta
quality-hint-lossless = PNG no tiene pérdida: la calidad no se aplica
quality-hint-low = Calidad baja, archivo más pequeño
quality-hint-balanced = Calidad equilibrada
quality-hint-high = Calidad alta, archivo más grande
resize-label = Redimensionar
resize-longest-side = Lado más largo
resize-width = Ancho
resize-height = Alto
resize-percent = Porcentaje
resize-placeholder = Tamaño
resize-keeps-ratio = Se mantienen las proporciones de la imagen
resize-no-upscale = No ampliar las imágenes más pequeñas
output-label = Carpeta de salida (opcional)
output-same-folder = Misma carpeta que el original
output-change = Cambiar la carpeta de salida
output-choose = Elegir una carpeta de salida
output-reset = Volver a la carpeta original
convert-button = Convertir las imágenes
convert-running = Convirtiendo…
convert-hint-empty = Añada imágenes para convertir
convert-hint-done = Todas las imágenes están procesadas: cambie un ajuste para volver a convertirlas

## Pantalla Historial

history-empty = Todavía no hay conversiones
history-title = Historial de conversiones
history-clear = Vaciar el historial
history-clear-question = ¿Vaciar todo el historial?
history-clear-cancel = Cancelar
history-clear-confirm = Vaciar
history-column-file = ARCHIVO
history-column-original = ORIGINAL
history-column-converted = CONVERTIDO
history-column-dimensions = DIMENSIONES
history-column-gain = AHORRO
history-column-date = FECHA
history-column-actions = ACCIONES

## Mensajes de la barra de estado

notice-preferences-not-saved = No se pudieron guardar las preferencias
notice-settings-not-saved = No se pudieron guardar los ajustes
notice-history-not-saved = No se pudo guardar el historial
notice-download-page = No se pudo abrir la página de descarga
notice-ignored-files =
    { $count ->
        [one] { $count } archivo ignorado
       *[other] { $count } archivos ignorados
    }: formato no compatible o archivo ilegible
notice-file-missing = El archivo ya no existe: { $name }
notice-cannot-open = No se pudo abrir { $name }
conversion-unexpected-error = Error inesperado durante la conversión
file-unknown = desconocido

## Selectores de archivos

dialog-choose-images = Elegir imágenes
dialog-images-filter = Imágenes
dialog-choose-output = Elegir la carpeta de salida

## Errores de conversión

error-invalid-path = La ruta del archivo de origen no es válida o falta
error-unsupported-extension = Extensión no compatible: { $extension }
error-no-extension = (ninguna)
error-source-not-found = El archivo de origen no existe: { $path }
error-not-a-file = La ruta no corresponde a un archivo: { $path }
error-invalid-quality = La calidad debe ser un número entre 1 y 100
error-invalid-resize = El tamaño solicitado debe estar entre 1 y { $max } { $unit }
error-too-large = La imagen mediría { $width } × { $height } px: el formato { $format } está limitado a { $max } px por lado
error-too-many-pixels = La imagen mediría { $width } × { $height } px: es demasiado para procesarla (máximo { $max } megapíxeles)
error-empty-output = El archivo de salida está vacío
error-decode = Imagen ilegible: { $error }
error-resize = Error al redimensionar: { $error }
error-encode = Error de codificación: { $error }
error-resize-buffer = búfer de salida no válido
error-webp-config = configuración WebP no válida
error-webp-version = versión de libwebp incompatible
error-webp-animation = no se pudo crear el codificador de animación
error-webp-dimensions = fotograma con dimensiones inesperadas
error-webp-unknown = error desconocido

## Línea de comandos y notificaciones

cli-usage = Uso:
cli-files = archivos…
cli-files-or-folders = archivos o carpetas…
cli-format = formato
cli-size = tamaño
cli-help-open = Abre la interfaz con estas imágenes
cli-help-open-gather = Igual, agrupando los lanzamientos simultáneos
cli-help-compress-format = Comprime con la calidad recomendada
cli-help-resize = Redimensiona con la calidad recomendada
cli-help-compress-saved = Comprime con los ajustes de la interfaz
cli-help-folders = Las carpetas se recorren con sus subcarpetas. Con --format y --size, las imágenes se guardan junto a las originales.
cli-help-formats = Formatos:
cli-help-original-format = formato original
cli-help-sizes = Tamaños (los formatos de redes sociales se recortan al centro y se guardan en JPEG):
cli-unknown-format = Formato desconocido: { $value }
cli-unknown-size = Tamaño desconocido: { $value }
cli-missing-size = Falta el tamaño: --size <tamaño>
report-compressed =
    { $count ->
        [one] { $count } imagen comprimida
       *[other] { $count } imágenes comprimidas
    }
report-resized =
    { $count ->
        [one] { $count } imagen redimensionada
       *[other] { $count } imágenes redimensionadas
    }
report-failures =
    { $count ->
        [one] , { $count } error
       *[other] , { $count } errores
    }
report-failure = { $name }: { $error }
report-impossible = No se puede comprimir
report-invalid-saved-resize = El tamaño de redimensionado guardado no es válido: corríjalo en Lotpix
report-no-images = No hay imágenes que procesar
report-supported-formats = Formatos compatibles: JPEG, PNG, GIF, TIFF y WebP

## Tamaños del menú contextual

size-4k = 4K (3840 px)
size-full-hd = Full HD (1920 px)
size-hd = HD (1280 px)
size-web = Web (1024 px)
size-thumbnail = Miniatura (640 px)
size-half = 50 %
size-instagram-square = Instagram cuadrado (1080 × 1080)
size-instagram-portrait = Instagram vertical (1080 × 1350)
size-story = Stories, Reels, TikTok (1080 × 1920)
size-facebook = Facebook, LinkedIn (1200 × 630)
size-x = X (1600 × 900)
size-youtube = Miniatura de YouTube (1280 × 720)
size-pinterest = Pinterest (1000 × 1500)
