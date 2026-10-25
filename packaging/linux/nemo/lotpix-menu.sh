#!/bin/sh
# Nemo n'a pas de sous-menus : les actions « Compresser avec Lotpix… » et « Redimensionner
# avec Lotpix… » font choisir le format ou la taille dans une liste, puis lancent Lotpix
# Utilisation : lotpix-menu.sh compress|resize <fichiers ou dossiers…>
kind=$1
shift

case $kind in
compress)
    choice=$(zenity --list --title="Compresser avec Lotpix" \
        --text="Format, avec la qualité recommandée :" \
        --column=id --column=Format --hide-column=1 --print-column=1 \
        --width=360 --height=320 \
        webp "WebP" \
        jpg "JPEG" \
        png "PNG (sans perte)" \
        avif "AVIF" \
        original "Format d'origine") || exit 0
    [ -n "$choice" ] && exec lotpix compress --format "$choice" "$@"
    ;;
resize)
    choice=$(zenity --list --title="Redimensionner avec Lotpix" \
        --text="Taille (les formats des réseaux sociaux sont recadrés et écrits en JPEG) :" \
        --column=id --column=Taille --hide-column=1 --print-column=1 \
        --width=420 --height=520 \
        4k "4K (3840 px)" \
        full-hd "Full HD (1920 px)" \
        hd "HD (1280 px)" \
        web "Web (1024 px)" \
        thumbnail "Miniature (640 px)" \
        half "50 %" \
        instagram-square "Instagram carré (1080 × 1080)" \
        instagram-portrait "Instagram portrait (1080 × 1350)" \
        story "Story, Reels, TikTok (1080 × 1920)" \
        facebook "Facebook, LinkedIn (1200 × 630)" \
        x "X (1600 × 900)" \
        youtube "Miniature YouTube (1280 × 720)" \
        pinterest "Pinterest (1000 × 1500)") || exit 0
    [ -n "$choice" ] && exec lotpix resize --size "$choice" "$@"
    ;;
esac
