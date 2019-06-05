#!/bin/bash


EWASM_NAME=ewasm_token
TNAME=wrc20ChallengeFiller.yml

make

./binary2hex $EWASM_NAME.wasm > $EWASM_NAME.hex

cat header.txt > $TNAME
echo "      code: '0x$(cat $EWASM_NAME.hex)'" >> $TNAME
cat footer.txt >> $TNAME

rm ~/workspace/tests2/src/GeneralStateTestsFiller/stEWASMTests/$TNAME
cp $TNAME ~/workspace/tests2/src/GeneralStateTestsFiller/stEWASMTests/$TNAME
cd ~/workspace/aleth/build/test/
ETHEREUM_TEST_PATH=/home/hugo/workspace/tests2/ ./testeth -t GeneralStateTests/stEWASMTests -- --filltests --vm /home/hugo/workspace/hera/build/src/libhera.so  --singlenet "Byzantium" --singletest wrc20Challenge
cd -

