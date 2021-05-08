from rest_framework import serializers, validators

from .models import HardwareSpec


class HardwareSpecSerializer(serializers.ModelSerializer):
    name = serializers.CharField(
        max_length=20,
        validators=[
            validators.UniqueValidator(queryset=HardwareSpec.objects.all())
        ],
    )
    num_registers = serializers.IntegerField(min_value=1, max_value=32)
    num_stacks = serializers.IntegerField(min_value=0, max_value=32)
    max_stack_length = serializers.IntegerField(min_value=0, max_value=256)

    class Meta:
        model = HardwareSpec
        exclude = ("slug",)
