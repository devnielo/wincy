import type { Dict } from "./en";

const es: Dict = {
  htmlLang: "es",
  ogLocale: "es_ES",
  languageName: "Español",
  meta: {
    title: "Wincy · El gestor de portapapeles para Windows que se maneja con el teclado",
    description:
      "Wincy guarda todo lo que copias y te lo devuelve con Alt+V. Búsqueda difusa, elementos fijados, pegado sin formato y privacidad de serie. Gratis y de código abierto para Windows 10 y 11.",
    logoAlt: "Logotipo de Wincy",
  },
  nav: {
    skip: "Saltar al contenido",
    label: "Principal",
    features: "Funciones",
    shortcuts: "Atajos",
    privacy: "Privacidad",
    install: "Instalación",
    faq: "Preguntas",
    language: "Idioma",
  },
  hero: {
    title: "Todo lo que copias, a un atajo de distancia.",
    lead: "Wincy recuerda lo que copias en Windows y te lo devuelve en una paleta que se abre junto al cursor. Pulsa Alt+V, escribe un par de letras y dale a Intro.",
    download: "Descargar para Windows",
    github: "Ver en GitHub",
    note: "Windows 10 y 11 · Gratis y de código abierto (MIT)",
    tryIt: "Pruébalo: escribe para filtrar, muévete con ↑ ↓ y pega con Intro.",
  },
  demo: {
    label: "Demo interactiva de Wincy",
    search: "Busca en tu portapapeles…",
    listLabel: "Historial del portapapeles",
    pinned: "Fijado",
    empty: "Nada coincide con tu búsqueda.",
    pasted: "Pegado",
    pastedPlain: "Pegado sin formato",
    closed: "Wincy está oculto, igual que al pulsar Esc en la aplicación.",
    reopen: "Abrir Wincy",
    kinds: {
      text: "Texto",
      url: "Enlace",
      color: "Color",
      code: "Código",
      image: "Imagen",
      files: "Archivos",
    },
    hints: [
      { keys: ["↑", "↓"], label: "Moverse" },
      { keys: ["Intro"], label: "Pegar" },
      { keys: ["Mayús", "Intro"], label: "Sin formato" },
      { keys: ["Alt", "1–9"], label: "Pegado rápido" },
      { keys: ["Esc"], label: "Cerrar" },
    ],
    items: [
      { kind: "text", text: "¡Gracias! Te envío el contrato firmado antes del viernes.", app: "outlook.exe", pinned: true },
      { kind: "code", text: "git log --oneline --graph --decorate -20", app: "WindowsTerminal.exe", pinned: true },
      { kind: "color", text: "#6D5BFF", app: "figma.exe" },
      { kind: "url", text: "https://learn.microsoft.com/es-es/windows/win32/dataxchg/clipboard", app: "msedge.exe" },
      { kind: "image", text: "Captura · 1280 × 720", app: "SnippingTool.exe" },
      { kind: "code", text: "export const debounce = (fn, ms) => { let t; … }", app: "Code.exe" },
      { kind: "files", text: "informe-T3.xlsx, factura-0423.pdf", app: "explorer.exe" },
      { kind: "text", text: "La reunión pasa a las 15:30, sala B2", app: "ms-teams.exe" },
      { kind: "color", text: "rgb(34, 184, 207)", app: "chrome.exe" },
      { kind: "url", text: "https://www.figma.com/design/brand-kit", app: "chrome.exe" },
      { kind: "text", text: "Tu pedido n.º 48213 ya está en camino", app: "chrome.exe" },
    ],
  },
  features: {
    title: "Una aplicación pequeña con herramientas afiladas",
    lead: "Todo en Wincy gira en torno a un momento: necesitas algo que copiaste antes, y lo necesitas ya.",
    items: [
      {
        icon: "palette",
        title: "Una paleta, no una ventana",
        body: "Alt+V abre una paleta de comandos compacta junto al puntero del ratón. Ya está cargada en segundo plano, así que aparece al instante, e Intro pega directamente en la aplicación que estabas usando. Pasa el ratón por un elemento para verlo completo en una vista previa flotante junto a la paleta, arrastra la paleta desde la barra de búsqueda y usa los botones si prefieres el ratón.",
      },
      {
        icon: "detect",
        title: "Sabe qué has copiado",
        body: "Reconoce enlaces, colores, código, imágenes y archivos, y los muestra como toca: muestras para los colores y miniaturas para las imágenes. Con Tab filtras por tipo.",
      },
      {
        icon: "privacy",
        title: "Privado por defecto",
        body: "Lo que copias desde gestores de contraseñas como Bitwarden, 1Password o KeePassXC nunca se guarda. Wincy respeta la marca con la que las aplicaciones dejan los datos sensibles fuera del historial (ExcludeClipboardContentFromMonitorProcessing), y puedes pausar la captura desde la bandeja del sistema.",
      },
      {
        icon: "plain",
        title: "Sin formato cuando quieras",
        body: "Mayús+Intro pega sin formato: el texto de una web llega a tu documento sin arrastrar sus fuentes, colores ni enlaces.",
      },
      {
        icon: "pin",
        title: "Fijados y Alt+1…9",
        body: "Fija lo que reutilizas (direcciones, respuestas, comandos) para que se quede arriba y nunca se borre. De Alt+1 a Alt+9 pegas los nueve primeros sin mirar.",
      },
      {
        icon: "search",
        title: "Búsqueda difusa",
        body: "Las erratas no importan. Busca por contenido, aplicación de origen y tipo, y encuentra en pocas teclas aquel enlace del martes pasado.",
      },
      {
        icon: "native",
        title: "Ligero y nativo",
        body: "Hecho con Rust y Tauri sobre el WebView del sistema, sin un navegador empaquetado dentro. Mica en Windows 11, Acrílico en Windows 10, tema claro y oscuro.",
      },
      {
        icon: "local",
        title: "Guardado en tu PC",
        body: "El historial vive en una base de datos SQLite local, dentro de tu perfil de usuario. Sin cuenta, sin nube, sin sincronización. Bórralo cuando quieras; lo fijado se queda.",
      },
      {
        icon: "update",
        title: "Siempre al día",
        body: "Wincy busca versiones nuevas en GitHub Releases y se actualiza solo. Cada actualización va firmada y se verifica antes de instalarse, y puedes desactivar la comprobación en Ajustes.",
      },
    ],
  },
  shortcuts: {
    title: "Atajos de teclado",
    lead: "Wincy está pensado para usarse con el teclado. Estos son todos sus atajos.",
    keysHeader: "Teclas",
    actionHeader: "Acción",
    typeLabel: "Escribe",
    rows: [
      { keys: ["Alt+V"], action: "Abrir Wincy desde cualquier aplicación (configurable)" },
      { keys: [], action: "Filtrar la lista con búsqueda difusa" },
      { keys: ["↑", "↓", "Re Pág", "Av Pág", "Inicio", "Fin"], action: "Mover la selección" },
      { keys: ["Intro"], action: "Pegar" },
      { keys: ["Mayús+Intro"], action: "Pegar sin formato" },
      { keys: ["Alt+1–9"], action: "Pegar el elemento visible número N (los fijados van primero)" },
      { keys: ["Ctrl+P"], action: "Fijar o dejar de fijar" },
      { keys: ["Ctrl+Supr"], action: "Eliminar el elemento" },
      { keys: ["Tab", "Mayús+Tab"], action: "Cambiar el filtro de tipo: todo, texto, enlaces, colores, código, imágenes, archivos" },
      { keys: ["Ctrl+Espacio"], action: "Mostrar u ocultar el panel de vista previa" },
      { keys: ["Ctrl+,"], action: "Abrir los ajustes" },
      { keys: ["Esc"], action: "Borrar la búsqueda o, si está vacía, cerrar Wincy" },
    ],
  },
  privacy: {
    title: "Tu portapapeles nunca sale de tu PC.",
    body: "Wincy no tiene telemetría, ni analíticas, ni cuentas. Todo lo que copias se queda en una base de datos en tu propio disco. La única conexión que hace es para buscar actualizaciones en GitHub Releases, y puedes desactivarla.",
    points: [
      "Sin telemetría ni analíticas",
      "Sin cuentas ni sincronización en la nube",
      "Ignora los gestores de contraseñas",
      "Pausa la captura desde la bandeja cuando quieras",
      "Código abierto: puedes leer cada línea",
    ],
  },
  compare: {
    title: "¿Y por qué no Win+V?",
    lead: "El historial del portapapeles de Windows cumple para lo último que has copiado. Wincy es para todo lo demás. No se pisan, así que si te gusta Win+V, puedes seguir usándolo.",
    caption: "Wincy frente al historial del portapapeles de Windows",
    featureHeader: "Función",
    rows: [
      ["Tamaño del historial", "1000 elementos por defecto, hasta 10 000", "25 elementos"],
      ["Tras reiniciar", "Todo sigue ahí", "Se borra, salvo lo fijado"],
      ["Búsqueda", "Búsqueda difusa mientras escribes", "No tiene"],
      ["Pegado rápido", "Alt+1 … Alt+9", "No tiene"],
      ["Filtros por tipo", "Texto, enlaces, colores, código, imágenes, archivos", "No tiene"],
      ["Sincronización en la nube", "Nunca: todo es local", "Opcional, con cuenta de Microsoft"],
    ],
  },
  install: {
    title: "Listo en un minuto",
    steps: [
      {
        title: "Descarga el instalador",
        body: "Baja el archivo que termina en -setup.exe de la última versión publicada en GitHub.",
        link: "Ir a la última versión",
      },
      {
        title: "Sortea SmartScreen",
        body: "Wincy todavía no está firmado digitalmente (los certificados cuestan un dinero que un proyecto de código abierto no tiene), así que Windows puede mostrar «Windows protegió su PC». Haz clic en:",
        moreInfo: "Más información",
        runAnyway: "Ejecutar de todas formas",
      },
      {
        title: "Se instala solo para ti",
        body: "No hacen falta permisos de administrador. Wincy se instala solo en tu cuenta y arranca con Windows. Puedes desactivarlo en Ajustes.",
      },
      {
        title: "Pulsa Alt+V",
        body: "Copia un par de cosas y pulsa Alt+V. Wincy vive en la bandeja del sistema; haz clic derecho en su icono para ver más opciones.",
      },
    ],
    note: "Todavía no está disponible en winget ni como versión portable (.zip).",
  },
  faq: {
    title: "Preguntas",
    items: [
      {
        q: "¿De verdad es gratis?",
        a: "Sí. Es código abierto con licencia MIT, sin versión de pago, sin anuncios y sin rastreo.",
      },
      {
        q: "¿Dónde se guarda mi historial?",
        a: "En una base de datos SQLite dentro de tu perfil de usuario de Windows (la carpeta de datos de la aplicación). En Ajustes hay un botón que la abre en el Explorador de archivos. Las imágenes copiadas se guardan ahí como archivos PNG.",
      },
      {
        q: "¿Guardará mis contraseñas?",
        a: "No si vienen de gestores de contraseñas que marcan lo que copian como sensible, ni de las aplicaciones de la lista de ignoradas: por defecto, Bitwarden, 1Password, KeePass, KeePassXC, Enpass, Dashlane, LastPass, NordPass y Proton Pass. Puedes añadir cualquier otra en Ajustes.",
      },
      {
        q: "¿Puedo cambiar el atajo Alt+V?",
        a: "Sí, en Ajustes. También puedes elegir cuántos elementos guardar, dejar de capturar imágenes o hacer que Intro copie en lugar de pegar.",
      },
      {
        q: "¿Funciona en Windows 10?",
        a: "Sí. En Windows 11 usa el material Mica y en Windows 10, Acrílico. En los dos tienes tema claro y oscuro.",
      },
      {
        q: "¿Por qué Windows me avisa al instalarlo?",
        a: "Porque el instalador no está firmado digitalmente, y SmartScreen marca las aplicaciones sin firma hasta que ganan reputación. El código fuente y el proceso de publicación son públicos en GitHub, así que puedes comprobar exactamente qué estás ejecutando.",
      },
      {
        q: "¿Cómo lo desinstalo?",
        a: "Como cualquier otra aplicación: abre Configuración → Aplicaciones, busca Wincy y elige Desinstalar.",
      },
    ],
  },
  footer: {
    license: "Gratis y de código abierto con licencia MIT.",
    source: "Código fuente",
    releases: "Versiones",
    issues: "Informar de un error",
  },
};

export default es;
