# !/usr/bin/env python3
import os
import re
import sys
import shutil


def main():
    scriptdir = os.path.dirname(os.path.realpath(__file__))
    lines = []
    encoding = 'utf-8'
    filename = os.path.join(scriptdir, "MaterialDesign-Webfont", "css", "materialdesignicons.css")
    pattern = ".*mdi.*\:\:before.*|.*content.*\\\F.*"
    matched = re.compile(pattern).search
    with open(filename, encoding=encoding) as input_file:
        for line in input_file:
            if matched(line):
                lines.append(line.strip())

    i = 0
    json = []
    while i < len(lines):
        name_match = re.match(".*(mdi-[a-z0-9-]*).*", lines[i])
        name = name_match.group(1)
        code_match = re.match(".*\"\\\(F[A-F0-9]*)\".*", lines[i + 1])
        code = code_match.group(1)
        print(name + code)
        json.append("    { \"name\": \"%s\", \"code\":\"%s\" }" % (name, code))
        i += 2

    jsonstr = "{\n  \"symbols\":[\n" + ",\n".join(json) + "\n  ]\n}"

    json_file = os.path.join(scriptdir, "..", "src", "main", "resources", "META-INF", "resources",
                             "MaterialDesignIcons.json")

    jsonfile = open(json_file, "w")
    jsonfile.write(jsonstr)
    jsonfile.close()

    shutil.copyfile(os.path.join(scriptdir, "MaterialDesign-Webfont", "fonts", "materialdesignicons-webfont.ttf"),
                    os.path.join(scriptdir, "..", "src", "main", "resources", "META-INF", "resources",
                                 "MaterialDesignIcons.ttf"))


main()
