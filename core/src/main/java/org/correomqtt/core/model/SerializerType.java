package org.correomqtt.core.model;

public enum SerializerType {
    BOOLEAN("org.apache.kafka.common.serialization.BooleanSerializer"),
    BYTE_ARRAY("org.apache.kafka.common.serialization.ByteArraySerializer"),
    BYTE_BUFFER("org.apache.kafka.common.serialization.ByteBufferSerializer"),
    BYTES("org.apache.kafka.common.serialization.BytesSerializer"),
    DOUBLE("org.apache.kafka.common.serialization.DoubleSerializer"),
    FLOAT("org.apache.kafka.common.serialization.FloatSerializer"),
    INTEGER("org.apache.kafka.common.serialization.IntegerSerializer"),
    LIST("org.apache.kafka.common.serialization.ListSerializer"),
    LONG("org.apache.kafka.common.serialization.LongSerializer"),
    SHORT("org.apache.kafka.common.serialization.ShortSerializer"),
    STRING("org.apache.kafka.common.serialization.StringSerializer"),
    UUID("org.apache.kafka.common.serialization.UUIDSerializer"),
    VOID("org.apache.kafka.common.serialization.VoidSerializer");
    private final String serializer;

    SerializerType(String serializer){
        this.serializer = serializer;
    }

    public String getSerializer(){
        return this.serializer;
    }
}
