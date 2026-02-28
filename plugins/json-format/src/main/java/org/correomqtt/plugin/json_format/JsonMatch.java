package org.correomqtt.plugin.json_format;

record JsonMatch(String type, int start, int end) {

    String getType() {
        return type;
    }

    int getStart() {
        return start;
    }

    int getEnd() {
        return end;
    }
}
