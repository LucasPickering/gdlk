from django.contrib import admin
from .models import HardwareSpec, Puzzle, Player, PlayerSolution


@admin.register(Player, PlayerSolution)
class CoreAdmin(admin.ModelAdmin):
    pass


@admin.register(HardwareSpec)
class HardwareSpecAdmin(admin.ModelAdmin):
    prepopulated_fields = {
        "slug": ("name",),
    }


@admin.register(Puzzle)
class PuzzleAdmin(admin.ModelAdmin):
    prepopulated_fields = {
        "slug": ("name",),
    }
