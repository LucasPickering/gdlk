import graphene
from graphene import relay, ObjectType
from graphene_django import DjangoObjectType, DjangoConnectionField

from .models import HardwareSpec, Puzzle, Player, PlayerSolution


class HardwareSpecNode(DjangoObjectType):
    class Meta:
        model = HardwareSpec
        interfaces = (relay.Node,)


class PuzzleNode(DjangoObjectType):
    class Meta:
        model = Puzzle
        interfaces = (relay.Node,)


class PlayerNode(DjangoObjectType):
    class Meta:
        model = Player
        interfaces = (relay.Node,)
        fields = ('id', 'username', 'puzzleSolutions')


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
