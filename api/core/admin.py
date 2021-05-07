from django.contrib import admin
from .models import HardwareSpec, ProgramSpec


@admin.register(HardwareSpec)
@admin.register(ProgramSpec)
class CoreAdmin(admin.ModelAdmin):
    pass
