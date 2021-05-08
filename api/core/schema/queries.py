import graphene
from graphene import relay, ObjectType
from graphene_django import DjangoObjectType, DjangoConnectionField

from ..models import HardwareSpec, Puzzle, Player, PlayerSolution


class HardwareSpecNode(DjangoObjectType):
    class Meta:
        model = HardwareSpec
        interfaces = (relay.Node,)


class PuzzleNode(DjangoObjectType):
    class Meta:
        model = Puzzle
        interfaces = (relay.Node,)
        # Don't think we'll need puzzle->player directly, so force caller to go
        # through PlayerSolution. We can add this back later if we need it
        exclude = ('players',)


class PlayerNode(DjangoObjectType):
    class Meta:
        model = Player
        interfaces = (relay.Node,)
        # Don't think we'll need player->puzzle directly, so force caller to go
        # through PlayerSolution. We can add this back later if we need it
        exclude = ("puzzles",)

    username = graphene.String(required=True)


class PlayerSolutionNode(DjangoObjectType):
    class Meta:
        model = PlayerSolution
        interfaces = (relay.Node,)


class Query(ObjectType):
    node = relay.Node.Field()
    player = relay.Node.Field(PlayerNode)
    hardware_spec = graphene.Field(
        HardwareSpecNode,
        description="Get a single hardware spec by its slug",
        slug=graphene.String(
            required=True,
            description="The unique slug of the hardware spec to fetch",
        ),
    )
    hardware_specs = DjangoConnectionField(HardwareSpecNode)
