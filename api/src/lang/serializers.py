from rest_framework import serializers


class CompileSourceSerializer(serializers.Serializer):
    source = serializers.CharField()


class CompileResultSerializer(serializers.Serializer):
    ast = serializers.CharField()
