import graphene
from graphene_django.rest_framework.mutation import (
    ClientIDMutation,
    MutationOptions,
)
from graphql_relay.connection.arrayconnection import offset_to_cursor
from graphql import GraphQLError

from .queries import PuzzleSolutionNode
from ..models import Player
from ..serializers import PuzzleSolutionSerializer
from ..util import get_node_field_name


class NodeMutationOptions(MutationOptions):
    """
    Meta-class options for NodeMutation base class
    """

    serializer_class = None
    node_class = None
    edge_class = None
    node_field = None
    edge_field = None


class NodeMutation(ClientIDMutation):
    """
    A mutation that creates, updates, or deletes some GraphQL node. This
    mutation is associated with a particular node class, and abstracts out a lot
    of boilerplate functionality for mutations around that class. It will
    automatically return the mutated node as well as an edge that wraps that
    node. The edge is useful for appending to (or removing from) existing
    connections in the Relay store in the UI.

    Behavior is configured through the `Meta` class:

    ### Required `Meta` Fields
    - `node_class`
    - `serializer_class`

    ### Optional `Meta` Fields
    - `node_field`
    - `edge_field`

    If not provided, the optional fields will be derived automatically
    """

    @classmethod
    def __init_subclass_with_meta__(
        cls, serializer_class=None, node_class=None, _meta=None, **options
    ):
        if not _meta:
            _meta = NodeMutationOptions(cls)
        _meta.serializer_class = serializer_class
        _meta.node_class = node_class
        # Derive edge class from the node class
        # https://github.com/graphql-python/graphene/issues/59#issuecomment-339648802
        _meta.edge_class = node_class._meta.connection.Edge

        # Automatically determine output fields based on the input class
        # (if not already provided by the user)
        if not _meta.node_field:
            _meta.node_field = get_node_field_name(_meta.node_class)
        if not _meta.edge_field:
            _meta.edge_field = f"{_meta.node_field}_edge"

        # Create the output fields
        _meta.fields = {
            _meta.node_field: graphene.Field(_meta.node_class, required=True),
            _meta.edge_field: graphene.Field(_meta.edge_class, required=True),
        }

        super().__init_subclass_with_meta__(_meta=_meta, **options)

    @classmethod
    def mutate_and_get_payload(cls, root, info, **input):
        player = Player.get_or_create_for_user(info.context.user)
        serializer = cls._meta.serializer_class(
            data={
                # Always pass in the player ID from the request, the serializer
                # may or may not use it. The subclass is free to override this
                # value by defining its own player_id input field.
                "player_id": player.id,
                **input,
            },
        )

        if serializer.is_valid():
            node = serializer.save()
            edge = cls._meta.edge_class(node=node, cursor=offset_to_cursor(0))
            # Populate two output fields: one of just the node, one of the
            # node wrapped in an edge.
            return cls(
                **{cls._meta.node_field: node, cls._meta.edge_field: edge}
            )
        else:
            raise GraphQLError(
                "Input validation error(s)", extensions=serializer.errors
            )


class SavePuzzleSolutionMutation(NodeMutation):
    """
    Create OR update a puzzle solution. Solutions are unique by their
    (name,player,puzzle) combo, so if a solution does not exist for that combo,
    a new one is created. If it does exist, the existing record is updated.
    """

    class Meta:
        node_class = PuzzleSolutionNode
        serializer_class = PuzzleSolutionSerializer

    class Input:
        puzzle_id = graphene.ID(required=True)
        name = graphene.String(required=True)
        source_code = graphene.String(required=True)


class Mutation(graphene.ObjectType):
    save_puzzle_solution = SavePuzzleSolutionMutation.Field()
