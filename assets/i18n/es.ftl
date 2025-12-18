# Application name term - single source of truth for branding
-app-name = IcedLens

window-title = { -app-name }
new-image-title = Nueva imagen
settings-back-to-viewer-button = Volver al Visor
settings-title = Configuración
settings-section-general = General
settings-section-display = Visualización
settings-section-video = Vídeo
settings-section-fullscreen = Pantalla completa
select-language-label = Seleccionar idioma:
language-name-en-US = Inglés
language-name-fr = Francés
language-name-es = Español
language-name-de = Alemán
language-name-it = Italiano
error-load-image-heading = No se pudo abrir la imagen.
error-details-show = Mostrar detalles
error-details-hide = Ocultar detalles
error-details-technical-heading = Detalles técnicos
viewer-zoom-label = Zoom
viewer-zoom-input-placeholder = 100
viewer-zoom-reset-button = Restablecer
viewer-fit-to-window-toggle = Ajustar a ventana
viewer-zoom-input-error-invalid = Por favor, ingrese un número válido.
viewer-zoom-step-error-invalid = El paso de zoom debe ser un número.
viewer-zoom-step-error-range = El paso de zoom debe estar entre 1% y 200%.
viewer-delete-tooltip = Eliminar la imagen actual
viewer-zoom-in-tooltip = Acercar
viewer-zoom-out-tooltip = Alejar
viewer-fullscreen-tooltip = Alternar pantalla completa
viewer-fullscreen-disabled-unsaved = Guarde o cancele los cambios primero
viewer-double-click = Doble clic
viewer-scroll-wheel = Rueda del ratón
viewer-click-drag = Clic + arrastrar
settings-zoom-step-label = Paso de zoom
settings-zoom-step-placeholder = 10
settings-zoom-step-hint = Elija cuánto cambia el zoom al usar los controles (1% a 200%).
settings-background-label = Fondo del visor
settings-background-light = Claro
settings-background-dark = Oscuro
settings-background-checkerboard = Tablero de ajedrez
settings-theme-mode-label = Tema de la aplicación
settings-theme-system = Seguir el sistema
settings-theme-light = Claro
settings-theme-dark = Oscuro
help-usage-heading = USO:
help-options-heading = OPCIONES:
help-args-heading = ARGUMENTOS:
help-examples-heading = EJEMPLOS:
help-line-option-help = -h, --help        Mostrar este texto de ayuda
help-line-option-lang =     --lang <id>    Establecer idioma (ej. en-US, fr)
help-arg-image-path = <RUTA>      Ruta a un archivo multimedia o directorio para abrir
help-example-1 = iced_lens ./foto.png
help-example-2 = iced_lens ./mis_fotos/
help-example-3 = iced_lens --lang fr ./imagen.jpg
help-description = { -app-name } – Visor de imágenes
help-line-option-i18n-dir =     --i18n-dir <ruta>  Cargar traducciones desde directorio
help-line-option-data-dir =     --data-dir <ruta>  Anular directorio de datos (archivos de estado)
help-line-option-config-dir =     --config-dir <ruta>  Anular directorio de configuración (settings.toml)
settings-sort-order-label = Orden de navegación de imágenes
settings-sort-alphabetical = Alfabético
settings-sort-modified = Fecha de modificación
settings-sort-created = Fecha de creación
settings-overlay-timeout-label = Retraso de ocultación automática en pantalla completa
settings-overlay-timeout-hint = Tiempo antes de que los controles desaparezcan en modo de pantalla completa.
seconds = segundos
image-editor-title = Editor de imágenes
image-editor-back-to-viewer = Volver al visor
image-editor-cancel = Cancelar
image-editor-save = Guardar
image-editor-save-as = Guardar como...
image-editor-tool-crop = Recortar
image-editor-tool-resize = Redimensionar
image-editor-tool-light = Luz
image-editor-rotate-section-title = Rotación
image-editor-rotate-right-tooltip = Rotar imagen en sentido horario
image-editor-rotate-left-tooltip = Rotar imagen en sentido antihorario
image-editor-flip-section-title = Voltear
image-editor-flip-horizontal-tooltip = Voltear imagen horizontalmente (espejo izquierda-derecha)
image-editor-flip-vertical-tooltip = Voltear imagen verticalmente (espejo arriba-abajo)
image-editor-resize-section-title = Redimensionar
image-editor-resize-scale-label = Escala
image-editor-resize-dimensions-label = Tamaño objetivo
image-editor-resize-width-label = Ancho (px)
image-editor-resize-height-label = Alto (px)
image-editor-resize-lock-aspect = Bloquear relación de aspecto
image-editor-resize-presets-label = Ajustes predefinidos
image-editor-resize-apply = Aplicar redimensionamiento
image-editor-light-section-title = Ajustes de luz
image-editor-light-brightness-label = Brillo
image-editor-light-contrast-label = Contraste
image-editor-light-reset = Restablecer
image-editor-light-apply = Aplicar
image-editor-crop-section-title = Recortar
image-editor-crop-ratio-label = Relación de aspecto
image-editor-crop-ratio-free = Libre
image-editor-crop-ratio-square = Cuadrado (1:1)
image-editor-crop-ratio-landscape = Horizontal (16:9)
image-editor-crop-ratio-portrait = Vertical (9:16)
image-editor-crop-ratio-photo = Foto (4:3)
image-editor-crop-ratio-photo-portrait = Foto vertical (3:4)
image-editor-crop-apply = Aplicar recorte
image-editor-undo-redo-section-title = Última modificación
image-editor-undo = Deshacer
image-editor-redo = Rehacer
image-editor-export-format-label = Formato de exportación
media-loading = Cargando...
settings-video-autoplay-label = Reproducción automática de vídeo
settings-video-autoplay-enabled = Activada
settings-video-autoplay-disabled = Desactivada
settings-video-autoplay-hint = Cuando está activada, los vídeos comienzan a reproducirse automáticamente al abrirse.
video-play-tooltip = Reproducir (Espacio)
video-pause-tooltip = Pausar (Espacio)
video-mute-tooltip = Silenciar (M)
video-unmute-tooltip = Activar sonido (M)
video-loop-tooltip = Repetir
video-capture-tooltip = Capturar fotograma actual
video-step-forward-tooltip = Avanzar un fotograma (.)
video-step-backward-tooltip = Retroceder un fotograma (,)
video-more-tooltip = Más opciones
settings-audio-normalization-label = Normalización de volumen de audio
settings-audio-normalization-enabled = Activada
settings-audio-normalization-disabled = Desactivada
settings-audio-normalization-hint = Nivela automáticamente el volumen de audio entre diferentes archivos multimedia para evitar cambios bruscos de volumen.
settings-frame-cache-label = Tamaño de caché de keyframes (para búsqueda)
settings-frame-cache-hint = Almacena keyframes de vídeo para acelerar el desplazamiento por la línea de tiempo y los saltos a momentos específicos. Los valores más altos almacenan más keyframes para una navegación más rápida. Los cambios se aplican al abrir un nuevo vídeo.
settings-frame-history-label = Tamaño del historial de fotogramas (para retroceder)
settings-frame-history-hint = Almacena los fotogramas mostrados recientemente para permitir retroceder fotograma por fotograma. Solo se usa al navegar manualmente por los fotogramas, no durante la reproducción normal.
settings-keyboard-seek-step-label = Paso de búsqueda con teclado
settings-keyboard-seek-step-hint = Tiempo a saltar al usar las teclas de flecha durante la reproducción de vídeo.
megabytes = MB
error-load-video-heading = No se pudo reproducir este vídeo.
error-load-video-general = Ocurrió un error al cargar el vídeo.
error-load-video-unsupported-format = Este formato de archivo no es compatible.
error-load-video-unsupported-codec = El códec de vídeo '{ $codec }' no es compatible en este sistema.
error-load-video-corrupted = El archivo de vídeo parece estar dañado o no es válido.
error-load-video-no-video-stream = No se encontró ninguna pista de vídeo en este archivo.
error-load-video-decoding-failed = Falló la decodificación del vídeo: { $message }
error-load-video-io = No se pudo leer este archivo. Verifique que aún existe y que tiene permiso para abrirlo.

# Navigation bar
menu-settings = Configuración
menu-help = Ayuda
menu-about = Acerca de
navbar-edit-button = Editar

# Help screen
help-title = Ayuda
help-back-to-viewer-button = Volver al visor

# Common labels
help-tools-title = Herramientas disponibles
help-shortcuts-title = Atajos de teclado
help-usage-title = Cómo usar

# ─────────────────────────────────────────────────────────────────────────────
# Viewer Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-viewer = Visor de imágenes y vídeos
help-viewer-role = Examine y visualice sus imágenes y vídeos. Navegue por los archivos en la misma carpeta y ajuste la visualización según sus preferencias.

help-viewer-tool-navigation = Navegación
help-viewer-tool-navigation-desc = Use los botones de flecha o el teclado para moverse entre archivos en la carpeta.
help-viewer-tool-zoom = Zoom
help-viewer-tool-zoom-desc = Desplácese con la rueda del ratón, use los botones +/- o ingrese un porcentaje directamente.
help-viewer-tool-pan = Desplazar
help-viewer-tool-pan-desc = Cuando esté ampliado, haga clic y arrastre la imagen para moverse.
help-viewer-tool-fit = Ajustar a ventana
help-viewer-tool-fit-desc = Escala automáticamente la imagen para que quepa completamente dentro de la ventana.
help-viewer-tool-fullscreen = Pantalla completa
help-viewer-tool-fullscreen-desc = Vista inmersiva con controles que se ocultan automáticamente (retraso configurable en Configuración).
help-viewer-tool-delete = Eliminar
help-viewer-tool-delete-desc = Eliminar permanentemente el archivo actual (se mueve a la papelera del sistema si está disponible).

help-viewer-key-navigate = Ir al archivo anterior/siguiente
help-viewer-key-edit = Abrir imagen en editor
help-viewer-key-fullscreen = Entrar/salir de pantalla completa
help-viewer-key-exit-fullscreen = Salir del modo de pantalla completa
help-viewer-key-info = Alternar panel de información del archivo

help-mouse-title = Interacciones con el ratón
help-viewer-mouse-doubleclick = Doble clic en imagen/vídeo para alternar pantalla completa
help-viewer-mouse-wheel = Acercar/alejar
help-viewer-mouse-drag = Desplazar imagen cuando esté ampliada

# ─────────────────────────────────────────────────────────────────────────────
# Video Playback Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-video = Reproducción de vídeo
help-video-role = Reproduzca archivos de vídeo con controles completos de reproducción. Ajuste el volumen, busque en la línea de tiempo y navegue fotograma por fotograma para un posicionamiento preciso.

help-video-tool-playback = Reproducir/Pausar
help-video-tool-playback-desc = Inicie o detenga la reproducción de vídeo con el botón de reproducción o la tecla Espacio.
help-video-tool-timeline = Línea de tiempo
help-video-tool-timeline-desc = Haga clic en cualquier lugar de la barra de progreso para saltar a esa posición.
help-video-tool-volume = Volumen
help-video-tool-volume-desc = Arrastre el control deslizante de volumen o haga clic en el icono del altavoz para silenciar/activar sonido.
help-video-tool-loop = Repetir
help-video-tool-loop-desc = Active para reiniciar automáticamente el vídeo cuando termine.
help-video-tool-stepping = Navegación por fotogramas
help-video-tool-stepping-desc = Cuando esté en pausa, avance o retroceda un fotograma a la vez para navegación precisa.
help-video-tool-capture = Captura de fotograma
help-video-tool-capture-desc = Guarde el fotograma actual del vídeo como archivo de imagen (se abre en el editor).

help-video-key-playpause = Reproducir o pausar el vídeo
help-video-key-mute = Alternar silencio de audio
help-video-key-seek = Buscar atrás/adelante (durante la reproducción)
help-video-key-volume = Aumentar/disminuir volumen
help-video-key-step-back = Retroceder un fotograma (cuando esté en pausa)
help-video-key-step-forward = Avanzar un fotograma (cuando esté en pausa)

# ─────────────────────────────────────────────────────────────────────────────
# Image Editor Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-editor = Editor de imágenes
help-editor-role = Realice ajustes en sus imágenes: rotar, recortar a un área específica o redimensionar a diferentes dimensiones.
help-editor-workflow = Todos los cambios no son destructivos hasta que los guarde. Use "Guardar" para sobrescribir el original, o "Guardar como" para crear un archivo nuevo y preservar el original.

help-editor-rotate-title = Rotación
help-editor-rotate-desc = Rote o voltee la imagen para corregir la orientación o crear efectos de espejo.
help-editor-rotate-left = Rotar 90° en sentido antihorario
help-editor-rotate-right = Rotar 90° en sentido horario
help-editor-flip-h = Voltear horizontalmente (espejo izquierda/derecha)
help-editor-flip-v = Voltear verticalmente (espejo arriba/abajo)

help-editor-crop-title = Recortar
help-editor-crop-desc = Elimine áreas no deseadas seleccionando la región que desea conservar.
help-editor-crop-ratios = Elija una relación predefinida (1:1 cuadrado, 16:9 horizontal, 9:16 vertical, 4:3 o 3:4 foto) o recorte libremente.
help-editor-crop-usage = Arrastre los manipuladores para ajustar la selección, luego haga clic en "Aplicar" para confirmar.

help-editor-resize-title = Redimensionar
help-editor-resize-desc = Cambie las dimensiones de la imagen para hacerla más grande o más pequeña.
help-editor-resize-scale = Escalar por porcentaje (ej. 50% para reducir a la mitad el tamaño)
help-editor-resize-dimensions = Ingrese ancho y alto exactos en píxeles
help-editor-resize-lock = Bloquear relación de aspecto para mantener las proporciones
help-editor-resize-presets = Use ajustes predefinidos para tamaños comunes (HD, Full HD, 4K...)

help-editor-light-title = Luz
help-editor-light-desc = Ajuste finamente el brillo y el contraste de su imagen.
help-editor-light-brightness = Brillo: aclare u oscurezca la imagen en general
help-editor-light-contrast = Contraste: aumente o disminuya la diferencia entre áreas claras y oscuras
help-editor-light-preview = Los cambios se previsualizan en tiempo real antes de aplicarlos

help-editor-save-title = Guardar
help-editor-save-overwrite = Guardar: sobrescribe el archivo original
help-editor-save-as = Guardar como: crea un nuevo archivo (elija ubicación y formato)

help-editor-key-save = Guardar cambios actuales
help-editor-key-undo = Deshacer último cambio
help-editor-key-redo = Rehacer cambio deshecho
help-editor-key-cancel = Cancelar todos los cambios y salir

# ─────────────────────────────────────────────────────────────────────────────
# Frame Capture Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-capture = Captura de fotograma de vídeo
help-capture-role = Extraiga cualquier fotograma de un vídeo y guárdelo como archivo de imagen. Perfecto para crear miniaturas o capturar momentos específicos.

help-capture-step1 = Reproduzca o navegue el vídeo hasta el fotograma deseado
help-capture-step2 = Pause el vídeo (use navegación por fotogramas para precisión)
help-capture-step3 = Haga clic en el botón de la cámara en los controles de vídeo
help-capture-step4 = El fotograma se abre en el editor — guarde como PNG, JPEG o WebP

help-capture-formats = Formatos de exportación compatibles: PNG (sin pérdida), JPEG (tamaño de archivo menor), WebP (formato moderno con buena compresión).

# ─────────────────────────────────────────────────────────────────────────────
# Sección de edición de metadatos
# ─────────────────────────────────────────────────────────────────────────────
help-section-metadata = Edición de metadatos
help-metadata-role = Vea y edite metadatos EXIF incrustados en sus archivos de imagen. Modifique información de la cámara, fecha de captura, coordenadas GPS y ajustes de exposición.

help-metadata-tool-view = Modo visualización
help-metadata-tool-view-desc = Vea información del archivo, detalles de la cámara, ajustes de exposición y coordenadas GPS en el panel de información.
help-metadata-tool-edit = Modo edición
help-metadata-tool-edit-desc = Haga clic en Editar para modificar los campos de metadatos. Los cambios se validan en tiempo real.
help-metadata-tool-save = Opciones de guardado
help-metadata-tool-save-desc = Guardar actualiza el archivo original, Guardar como crea una copia con los nuevos metadatos.

help-metadata-fields-title = Campos editables
help-metadata-field-camera = Marca y modelo de cámara
help-metadata-field-date = Fecha de captura (formato EXIF)
help-metadata-field-exposure = Tiempo de exposición, apertura, ISO
help-metadata-field-focal = Distancia focal y equivalente 35mm
help-metadata-field-gps = Latitud y longitud GPS

help-metadata-note = Nota: La edición de metadatos solo está disponible para imágenes. La edición de metadatos de vídeo está prevista para una futura versión.

# About screen
about-title = Acerca de
about-back-to-viewer-button = Volver al visor

about-section-app = Aplicación
about-app-name = { -app-name }
about-app-description = Visor ligero de imágenes y vídeos con edición básica de imágenes.

about-section-license = Licencia
about-license-name = Mozilla Public License 2.0 (MPL-2.0)
about-license-summary = Copyleft a nivel de archivo: los archivos modificados deben compartirse bajo la misma licencia. Compatible con código propietario.

about-section-icon-license = Licencia de iconos
about-icon-license-name = Licencia de iconos de { -app-name }
about-icon-license-summary = Todos los iconos (logotipo de la aplicación e iconos de la interfaz) solo pueden redistribuirse sin modificar para representar { -app-name }.

about-section-credits = Créditos
about-credits-iced = Construido con el kit de herramientas GUI Iced
about-credits-ffmpeg = Reproducción de vídeo con FFmpeg
about-credits-fluent = Internacionalización por Project Fluent

about-section-links = Enlaces
about-link-repository = Código fuente
about-link-issues = Reportar problemas

# Notifications
notification-save-success = Imagen guardada exitosamente
notification-save-error = Error al guardar la imagen
notification-frame-capture-success = Fotograma capturado exitosamente
notification-frame-capture-error = Error al capturar fotograma
notification-delete-success = Archivo eliminado exitosamente
notification-delete-error = Error al eliminar archivo
notification-config-save-error = Error al guardar la configuración
notification-config-load-error = Error al cargar la configuración, usando valores predeterminados
notification-state-parse-error = Error al leer el estado de la aplicación, usando valores predeterminados
notification-state-read-error = Error al abrir el archivo de estado de la aplicación
notification-state-path-error = No se puede determinar la ruta de datos de la aplicación
notification-state-dir-error = Error al crear el directorio de datos de la aplicación
notification-state-write-error = Error al guardar el estado de la aplicación
notification-state-create-error = Error al crear el archivo de estado de la aplicación
notification-scan-dir-error = Error al escanear el directorio
notification-editor-frame-error = Error al abrir el editor con el fotograma capturado
notification-editor-create-error = Error al abrir el editor de imágenes
notification-editor-load-error = Error al cargar la imagen para editar
notification-video-editing-unsupported = La edición de vídeo aún no es compatible

# Metadata panel
metadata-panel-title = Información del archivo
metadata-panel-close = Cerrar panel
metadata-panel-close-disabled = Guarde o cancele los cambios primero
metadata-section-file = Archivo
metadata-section-camera = Cámara
metadata-section-exposure = Exposición
metadata-section-video = Vídeo
metadata-section-audio = Audio
metadata-section-gps = Ubicación
metadata-label-dimensions = Dimensiones
metadata-label-file-size = Tamaño de archivo
metadata-label-format = Formato
metadata-label-date-taken = Fecha de captura
metadata-label-camera = Cámara
metadata-label-exposure = Exposición
metadata-label-aperture = Apertura
metadata-label-iso = ISO
metadata-label-focal-length = Distancia focal
metadata-label-gps = Coordenadas
metadata-label-codec = Códec
metadata-label-bitrate = Tasa de bits
metadata-label-duration = Duración
metadata-label-fps = Fotogramas por segundo
metadata-value-unknown = Desconocido

# Edición de metadatos
metadata-edit-button = Editar
metadata-edit-disabled-video = La edición de metadatos no está disponible para vídeos
metadata-cancel-button = Cancelar
metadata-save-button = Guardar
metadata-save-as-button = Guardar como...
metadata-save-warning = Guardar modificará el archivo original
metadata-label-make = Marca
metadata-label-model = Modelo
metadata-label-focal-length-35mm = Distancia focal (35mm)
metadata-label-flash = Flash
metadata-label-latitude = Latitud
metadata-label-longitude = Longitud
metadata-validation-date-format = Formato: AAAA:MM:DD HH:MM:SS
metadata-validation-date-invalid = Valores de fecha/hora inválidos
metadata-validation-exposure-format = Formato: 1/250 o 0.004
metadata-validation-aperture-format = Formato: f/2.8 o 2.8
metadata-validation-iso-positive = Debe ser un número entero positivo
metadata-validation-focal-format = Formato: 50 mm o 50
metadata-validation-lat-range = Debe estar entre -90 y 90
metadata-validation-lon-range = Debe estar entre -180 y 180
metadata-validation-invalid-number = Número inválido

# Notificaciones de metadatos
notification-metadata-save-success = Metadatos guardados correctamente
notification-metadata-save-error = Error al guardar los metadatos
notification-metadata-validation-error = Por favor corrija los errores de validación antes de guardar

# Divulgación progresiva de metadatos
metadata-add-field = Añadir campo de metadatos...
metadata-no-fields-message = Sin campos de metadatos. Use "Añadir campo de metadatos" para agregar campos.

navbar-info-button = Info

# Empty state (no media loaded)
empty-state-title = Sin contenido multimedia
empty-state-subtitle = Arrastra archivos aquí o haz clic para abrir
empty-state-button = Abrir archivo
empty-state-drop-hint = Arrastra y suelta imágenes o vídeos en cualquier lugar

# Additional notifications
notification-empty-dir = No se encontraron archivos multimedia compatibles en esta carpeta
notification-load-error-io = No se pudo abrir el archivo. Verifica que existe y tienes permisos.
notification-load-error-svg = No se pudo renderizar el SVG. El archivo puede estar malformado.
notification-load-error-video = No se pudo reproducir el vídeo. El formato puede no ser compatible.
notification-load-error-timeout = La carga ha expirado. El archivo puede ser demasiado grande o el sistema está ocupado.
