# copy all ./contracts/<contractname>/<contractname>.json files to ./schemas/<contractname>.json

echo "Collecting schemas..."

mkdir -p ./schemas

for contract in ./contracts/*; do
    if [ -d "$contract" ]; then
        contractname=$(basename $contract)
        echo "Copying $contractname schema..."
        echo "cp $contract/$contractname.json ./schemas/$contractname/$contractname.json"
        mkdir -p ./schemas/$contractname
        cp $contract/schema/$contractname.json ./schemas/$contractname/$contractname.json
    fi
done