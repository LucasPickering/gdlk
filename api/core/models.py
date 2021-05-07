from django.db import models
from .fields import UUIDField


class HardwareSpec(models.Model):
    id = UUIDField()
    name = models.CharField(max_length=100)

    def __str__(self):
        return self.name


class ProgramSpec(models.Model):
    id = UUIDField()
    name = models.CharField(max_length=100)
    hardware_spec = models.ForeignKey(
        HardwareSpec, on_delete=models.CASCADE, related_name="program_specs"
    )

    def __str__(self):
        return self.name
