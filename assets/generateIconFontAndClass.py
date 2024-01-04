# Originally taken from https://github.com/jviksne/svg2webfont/blob/main/svg2webfont.py

import fontforge
import sys
import os
import xml.etree.ElementTree as ET
import re


class Rect:
    def __init__(self,
                 x1: float,
                 y1: float,
                 x2: float = None,
                 y2: float = None,
                 width: float = None,
                 height: float = None):

        if x2 is None:
            if width is None:
                raise Exception("Neither width nor x2 passed to Rect")
            x2 = x1 + width
        if y2 is None:
            if height is None:
                raise Exception("Neither height nor y2 passed to Rect")
            y2 = y1 + height

        self.min_x = min(x1, x2)
        self.min_y = min(y1, y2)
        self.max_x = max(x1, x2)
        self.max_y = max(y1, y2)

    @staticmethod
    def from_rect(rect):
        return Rect(
            x1=rect.min_x,
            y1=rect.min_y,
            x2=rect.max_x,
            y2=rect.max_y)

    @property
    def width(self):
        return self.max_x - self.min_x

    @property
    def height(self):
        return self.max_y - self.min_y

    def transform(self, matrix):
        new_xywh = transform(matrix, [self.min_x, self.min_y, self.width, self.height])
        return Rect(x1=new_xywh[0], y1=new_xywh[1], width=new_xywh[2], height=new_xywh[3])

    def move_to(self, x: float, y: float):
        self.min_x = x
        self.min_y = y

    def __repr__(self):
        return f"Rect(min_x={self.min_x}, min_y={self.min_y}, max_x={self.max_x}, max_y={self.max_y}, width={self.width}, height={self.height})"


def assert_dst_file_path(filepath: str, param: str):
    (path, file) = os.path.split(filepath)

    if file == '':
        print('%s does not point to a file' % (param,))
        sys.exit(-1)

    if path != '' and not os.path.isdir(path):
        print('directory %s of %s does not exist or is not a valid directory' % (path, param,))
        sys.exit(-1)


def parse_int_param(param, s: str | int):
    try:
        return int(s)
    except ValueError:
        print('bad %s value: %s' % (param, s))
        sys.exit(-1)


def parse_float_param(param, s: str | float | int):
    try:
        return float(s)
    except ValueError:
        print('bad %s value: %s' % (param, s))
        sys.exit(-1)


def get_rel_path(from_file: str, to_file: str, to_url: bool):
    (from_path, _) = os.path.split(from_file)
    (to_path, to_filename) = os.path.split(to_file)

    path = os.path.relpath(os.path.abspath(to_path), os.path.abspath(from_path))

    if to_url:
        return join_url_path(path.replace('\\', '/'), to_filename)

    return os.path.join(path, to_file)


def join_url_path(path: str, file: str):
    if path == '':
        return file

    if path.endswith('/'):
        return path + file

    return path + '/' + file


def get_svg_viewbox(path):
    # SVG viewbox has the origin on top left with positive values going right and down
    tree = ET.parse(path)
    root = tree.getroot()
    if 'viewBox' in root.attrib:
        parts = root.attrib['viewBox'].split()
        if len(parts) == 4:
            return Rect(x1=float(parts[0]), y1=float(parts[1]), width=float(parts[2]), height=float(parts[3]))
    if 'width' in root.attrib and 'height' in root.attrib:
        return Rect(x1=0.0, y1=0.0, width=float(root.attrib['width']), height=float(root.attrib['height']))
    return None


def get_glyph_bbox_rect(glyph):
    rect = glyph.boundingBox()  # xmin,ymin, xmax,ymax from baseline to ascender
    return Rect(x1=rect[0], y1=rect[1], x2=rect[2], y2=rect[3])


def transformXY(matrix, xy):
    a, b, c, d, e, f = matrix
    x, y = xy

    x1 = a * x + c * y + e
    y1 = b * x + d * y + f
    return (x1, y1)


def transform(matrix, xywh):
    x1, y1 = transformXY(matrix, (xywh[0], xywh[1]))
    x2, y2 = transformXY(matrix, (xywh[0] + xywh[2], xywh[1] + xywh[3]))
    return (x1, y1, x2 - x1, y2 - y1)


def is_glyph_empty(glyph):
    if len(glyph.layers) < 2:
        return True
    layer = glyph.layers[1]
    if len(layer) == 0:
        return True
    return False


def create_font_and_class(svg_dir, font_file, json_file, prefix, font_name):
    font = fontforge.font()

    font.em = 4096
    font.ascent = 1000
    font.descent = 100

    next_unicode = int("F0001", 16)

    if not svg_dir.endswith('/'):
        svg_dir += '/'
    svg_files = os.listdir(svg_dir)
    svg_files = [f for f in svg_files if f.endswith('.svg')]
    svg_files.sort()

    max_width = 0.0
    max_height = 0.0

    json = []

    for svg_file in svg_files:

        glyph_name = svg_file[0:-len('.svg')]
        if len(glyph_name) == 0:
            continue

        curr_unicode = next_unicode
        next_unicode = curr_unicode + 1
        glyph = font.createChar(curr_unicode)
        svg_file_path = os.path.join(svg_dir, svg_file)
        try:
            glyph.importOutlines(svg_file_path, scale=False)
        except Exception as e:
            print(f"Failed to import outlines from SVG file %s: {e}" % (svg_file_path,))
            continue

        if is_glyph_empty(glyph):
            print(
                f"Warning: The SVG file '{svg_file_path}' is either empty or FontForge failed to import outlines from it.")

        glyph.glyphname = glyph_name

        svg_viewbox = get_svg_viewbox(svg_file_path)

        bbox = get_glyph_bbox_rect(glyph)

        if svg_viewbox == None:
            glyph_viewbox = Rect.from_rect(bbox)
        else:
            glyph_viewbox = Rect(
                x1=svg_viewbox.min_x,
                y1=float(font.ascent) - svg_viewbox.min_y,  # -960 in SVG should become 800 + 960 in glyph
                width=svg_viewbox.width,
                height=-svg_viewbox.height
            )
        scale = float(font.em) / max(glyph_viewbox.width, glyph_viewbox.height)
        matrix = (scale, 0, 0, scale, 0, 0)
        glyph.transform(matrix)
        glyph_viewbox = glyph_viewbox.transform(matrix)
        bbox = get_glyph_bbox_rect(glyph)
        adv_min_width = font.em
        adv_max_width = None

        advance_width = bbox.width
        if adv_min_width != None and advance_width < adv_min_width:
            advance_width = adv_min_width

        if adv_max_width != None and advance_width > adv_max_width:
            advance_width = adv_max_width

        x_move = (float(advance_width) - glyph_viewbox.width) / 2.0 - glyph_viewbox.min_x
        center = (float(font.ascent) + float(font.descent)) / 2.0 - font.descent
        y_move = -glyph_viewbox.max_y + center + glyph_viewbox.height / 2.0
        matrix = (1, 0, 0, 1, x_move, y_move)
        glyph.transform(matrix)
        glyph.width = int(round(advance_width))
        bbox = get_glyph_bbox_rect(glyph)

        json.append("    { \"name\": \"%s-%s\", \"code\":\"%s\" }" % (prefix, glyph_name, hex(curr_unicode)[2:].upper()))

    jsonstr = "{\n  \"symbols\":[\n" + ",\n".join(json) + "\n  ]\n}"

    jsonfile = open(json_file, "w")
    jsonfile.write(jsonstr)
    jsonfile.close()

    # Set the font's encoding to Unicode
    font.encoding = 'unicode'

    font.fontname = font_name
    font.fullname = font_name
    font.familyname = font_name

    font.generate(font_file)

    # Close the font
    font.close()


scriptdir = os.path.dirname(os.path.realpath(__file__))

create_font_and_class(
    os.path.join(scriptdir, "icons"),
    os.path.join(scriptdir, "..", "src", "main", "resources", "META-INF", "resources", "CorreoIcons.ttf"),
    os.path.join(scriptdir, "..", "src", "main", "resources", "META-INF", "resources", "CorreoIcons.json"),
    "correo",
    "CorreoIcons")

create_font_and_class(
    os.path.join(scriptdir, "MaterialDesign", "svg"),
    os.path.join(scriptdir, "..", "src", "main", "resources", "META-INF", "resources", "MaterialDesignIcons.ttf"),
    os.path.join(scriptdir, "..", "src", "main", "resources", "META-INF", "resources", "MaterialDesignIcons.json"),
    "mdi",
    "MaterialDesignIcons")
