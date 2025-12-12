#!/bin/bash
# Script pour retirer le délai de test de load_media()

FILE="src/media/mod.rs"

# Vérifier si le délai est présent
if ! grep -q "TEMPORARY.*test loading" "$FILE"; then
    echo "⚠️  Aucun délai de test trouvé dans $FILE"
    exit 1
fi

# Retirer via git
git checkout "$FILE"

echo "✅ Délai de test retiré de $FILE"
