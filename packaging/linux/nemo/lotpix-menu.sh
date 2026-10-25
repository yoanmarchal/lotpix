#!/bin/sh
# Nemo has no submenus: the "Compress with Lotpix…" and "Resize with Lotpix…" actions
# let the user pick the format or the size in a list, then run Lotpix.
# The list comes from Lotpix (`lotpix menu-choices`), in the language chosen in the
# application: title, prompt, then each value followed by its label
# Usage: lotpix-menu.sh compress|resize <files or folders…>
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

# zenity reads the cells of the list from its input, one per line
choice=$(printf '%s\n' "$choices" | tail -n +3 | zenity --list --title="$title" \
    --text="$prompt" --column=value --column=label --hide-column=1 --print-column=1 \
    --hide-header --width=420 --height=520) || exit 0
[ -n "$choice" ] && exec lotpix "$kind" "$option" "$choice" "$@"
