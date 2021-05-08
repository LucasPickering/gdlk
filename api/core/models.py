from django.core import validators
from django.contrib.auth.models import User
from django.db import models
from django.contrib.postgres import fields as pg_fields
from django.utils.functional import cached_property
from django.utils.text import slugify
from .fields import UUIDField


class HardwareSpec(models.Model):
    """
    A hardware spec defines a single piece of "hardware" that can execute GDLK
    programs (this isn't real hardware, just simulated hardware within the
    GDLK VM). Each hardware spec has multiple different capabilities that
    define how much flexibility the user has when writing and executing programs
    on that hardware.
    """

    id = UUIDField()
    name = models.CharField(max_length=50, blank=False, unique=True)
    slug = models.SlugField(max_length=50, blank=False, unique=True)
    num_registers = models.PositiveSmallIntegerField(
        validators=[validators.MinValueValidator(1)]
    )
    num_stacks = models.PositiveSmallIntegerField()
    max_stack_length = models.PositiveSmallIntegerField()

    def __str__(self):
        return self.name

    def save(self, *args, **kwargs):
        self.slug = slugify(self.name, allow_unicode=True)
        return super().save(*args, **kwargs)


class Puzzle(models.Model):
    """
    A GDLK puzzle. Puzzles have a predefined array of input values (integers),
    and a predefined expected output. The user's goal is to write a program
    that consumes the entire input and emits the expected output. A user can
    create multiple solutions to a single puzzle, which are stored in the
    PlayerSolution model.
    """

    id = UUIDField()
    name = models.CharField(max_length=50, blank=False, unique=True)
    slug = models.SlugField(max_length=50, blank=False, unique=True)
    description = models.TextField()
    hardware_spec = models.ForeignKey(
        HardwareSpec, on_delete=models.CASCADE, related_name="puzzles"
    )
    input = pg_fields.ArrayField(
        models.IntegerField(), validators=[validators.MaxLengthValidator(256)]
    )
    expected_output = pg_fields.ArrayField(
        models.IntegerField(), validators=[validators.MaxLengthValidator(256)]
    )

    def __str__(self):
        return self.name


class Player(models.Model):
    """
    A player is a single person that interacts with the platform. Players are
    different from users because User is a django-defined model that represents
    someone who has authenticated with the platform. Players haven't necessarily
    authed though -- they may be playing anonymously. Therefore, this model
    has a nullable one-to-one field with User. It will be populated for authed
    players, unpopulated for anonymous.
    """

    id = UUIDField()  # This is SAFE to share, like any other primary key
    user = models.OneToOneField(to=User, null=True, on_delete=models.CASCADE)
    # All the puzzles for which this player has created one solution
    puzzles = models.ManyToManyField(
        Puzzle, through="PlayerSolution", related_name="players"
    )

    @cached_property
    def username(self):
        if self.user:
            return self.user.username
        return "Anonymous User #77"  # TODO

    def __str__(self):
        return str(self.user) if self.user else f"Anonymous Player {self.id}"


class PlayerSolution(models.Model):
    id = UUIDField()
    puzzle = models.ForeignKey(
        Puzzle, on_delete=models.CASCADE, related_name="player_solutions"
    )
    player = models.ForeignKey(
        Player, on_delete=models.CASCADE, related_name="puzzle_solutions"
    )
    name = models.CharField(max_length=50, blank=False)
    source_code = models.TextField()
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        # A player can have multiple solutions to the same puzzle, but each one
        # must have a unique name
        unique_together = ("puzzle", "player", "name")
