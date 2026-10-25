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
