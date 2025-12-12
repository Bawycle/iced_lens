#!/bin/bash
# Script pour ajouter un délai de test dans load_media()

FILE="src/media/mod.rs"
LINE_NUM=183

# Vérifier si le délai est déjà présent
if grep -q "TEMPORARY.*test loading" "$FILE"; then
    echo "⚠️  Le délai de test est déjà présent dans $FILE"
    exit 1
fi

# Ajouter le délai après la ligne de détection du media_type
sed -i "${LINE_NUM}a\\
\\
    // TEMPORARY: Artificial delay to test loading indicator\\
    // Remove this before committing!\\
    std::thread::sleep(std::time::Duration::from_secs(2));" "$FILE"

echo "✅ Délai de 2 secondes ajouté dans $FILE à la ligne $LINE_NUM"
echo "Pour supprimer: git checkout $FILE"
