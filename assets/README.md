# Assets

## Custom Icons

Custom Icons are alle SVGs in `icons`. 
Correo will use them as icon font with a json mapping. These files can be generated with `generateCustomIcons.py`.

In order to run you have to install the python fontforge library. This is not possible via pip. Basically fontforge will install the python library automatically.
On debian based system there is also a package `python3-fontforge`. Unfortunately this does not work with virtual python environments.

Easiest way is to use python from the system:
```
/usr/bin/pythojn generateCustomIcons.py
```

This will create two files:
```
src/main/resources/META-INF/resources/CorreoIcons.ttf
src/main/resources/META-INF/resources/CorreoIcons.json
```

From now you are able to use the icons as usual.

## Material Design Icons

For available Icons see here: https://pictogrammers.com/library/mdi/

Checkout Material Design Icons
```
git clone https://github.com/Templarian/MaterialDesign-Webfont
```

Run `extractMapping.py`, which will create the mapping json file and copy the ttf. No need to install fontforge for this step.

```
src/main/resources/META-INF/resources/MaterialDesignIcons.ttf
src/main/resources/META-INF/resources/MaterialDesignIcons.json
```

From now all the material design icons are available as usual.

It is also possible to checkout the Material Design Icons from https://github.com/Templarian/MaterialDesign.git and generate the font yourself with `generateCustomIcons.py`. But currently not all icons are correctly imported via fontforge, so using the original ttf is the better option.