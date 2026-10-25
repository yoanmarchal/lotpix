#!/bin/sh
# Nemo n'a pas de sous-menus : les actions « Compresser avec Lotpix… » et « Redimensionner
# avec Lotpix… » font choisir le format ou la taille dans une liste, puis lancent Lotpix.
# La liste vient de Lotpix (`lotpix menu-choices`), dans la langue choisie dans l'application :
# titre, consigne, puis chaque valeur suivie de son libellé
# Utilisation : lotpix-menu.sh compress|resize <fichiers ou dossiers…>
kind=$1
shift

case $kind in
compress) option=--format ;;
resize) option=--size ;;
*) exit 1 ;;
esac

choices=$(lotpix menu-choices "$kind") || exit 1
title=$(printf '%s\n' "$choices" | sed -n 1p)
prompt=$(printf '%s\n' "$choices" | sed -n 2p)

# zenity lit les cellules de la liste sur son entrée, une par ligne
choice=$(printf '%s\n' "$choices" | tail -n +3 | zenity --list --title="$title" \
    --text="$prompt" --column=value --column=label --hide-column=1 --print-column=1 \
    --hide-header --width=420 --height=520) || exit 0
[ -n "$choice" ] && exec lotpix "$kind" "$option" "$choice" "$@"
