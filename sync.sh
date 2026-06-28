SRC="/media/sf_WallpaperEngine-Linux"
DEST="$HOME/wallpaperengine-local"

while true; do
  rsync -a --delete --exclude node_modules --exclude target --exclude .git "$SRC/" "$DEST/"
  sleep 5
done