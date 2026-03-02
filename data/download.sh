#!/bin/bash

URL="http://download.sgjp.pl/morfeusz/20251116/polimorf-20251116.tab.gz"
FILE="polimorf-20251116.tab.gz"

echo "Downloading PoliMorf dictionary..."
curl -O $URL

# gunzip removes the .gz file automatically after successful extraction
echo "Unpacking..."
gunzip $FILE

echo "Done! The file 'polimorf-20251116.tab' is ready for use."