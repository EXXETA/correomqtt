#/bin/bash

iconutil -c icns icon.iconset -o Icon.icns
mv Icon.icns ../src/main/deploy/package/

convert ico/Icon_16x16.png ico/Icon_24x24.png ico/Icon_32x32.png ico/Icon_48x48.png ico/Icon_64x64.png ico/Icon_72x72.png ico/Icon_80x80.png ico/Icon_96x96.png ico/Icon_128x128.png ico/Icon_256x256.png ico/Icon_512x512.png ico/Icon_1024x1024.png Icon.ico
mv Icon.ico ../src/main/deploy/package/

cp ico/Icon_512x512.png ../src/main/deploy/package/Icon.png
