import graphene
from graphene import relay, ObjectType
from graphene.relay.connection import Connection
from graphene_django import DjangoObjectType, DjangoConnectionField

from ..models import HardwareSpec, Puzzle, Player, PuzzleSolution


class ExtendedConnection(Connection):
    """
    TODO
    """

    class Meta:
        abstract = True

    total_count = graphene.Int()

    def resolve_total_count(root, info, **kwargs):
        return 5  # TODO


class HardwareSpecNode(DjangoObjectType):
    class Meta:
        model = HardwareSpec
        interfaces = (relay.Node,)
        connection_class = ExtendedConnection


class PuzzleSolutionNode(DjangoObjectType):
    class Meta:
        model = PuzzleSolution
        interfaces = (relay.Node,)
        connection_class = ExtendedConnection


class PuzzleNode(DjangoObjectType):
    class Meta:
        model = Puzzle
        interfaces = (relay.Node,)
        connection_class = ExtendedConnection
        # Don't think we'll need puzzle->player directly, so force caller to go
        # through PuzzleSolution. We can add this back later if we need it
        exclude = ("players", "player_solutions")

    # Rename player_solutions->puzzle_solutions, to match PuzzleSolution name
    puzzle_solutions = DjangoConnectionField(
        PuzzleSolutionNode, source="player_solutions"
    )
    puzzle_solution = graphene.Field(
        PuzzleSolutionNode,
        description="Get a single solution for this puzzle by its name",
        name=graphene.String(
            required=True,
            description="The unique name of the solution to fetch",
        ),
    )


class PlayerNode(DjangoObjectType):
    class Meta:
        model = Player
        interfaces = (relay.Node,)
        connection_class = ExtendedConnection
        # Don't think we'll need player->puzzle directly, so force caller to go
        # through PuzzleSolution. We can add this back later if we need it
        exclude = ("puzzles",)

    username = graphene.String(required=True)


class Query(ObjectType):
    node = relay.Node.Field()
    player = relay.Node.Field(PlayerNode)
    current_player = graphene.Field(PlayerNode, required=True)
    hardware_spec = graphene.Field(
        HardwareSpecNode,
        description="Get a single hardware spec by its slug",
        slug=graphene.String(
            required=True,
            description="The unique slug of the hardware spec to fetch",
        ),
    )
    hardware_specs = DjangoConnectionField(HardwareSpecNode)
    puzzle = graphene.Field(
        PuzzleNode,
        description="Get a single puzzle by its slug",
        slug=graphene.String(
            required=True,
            description="The unique slug of the puzzle to fetch",
        ),
    )
    puzzles = DjangoConnectionField(PuzzleNode)
