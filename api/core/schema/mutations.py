from django.db import IntegrityError
import graphene
from graphene_django.rest_framework.mutation import (
    ClientIDMutation,
    MutationOptions,
)
from graphql_relay import from_global_id
from graphql_relay.connection.arrayconnection import offset_to_cursor
from graphql import GraphQLError

from .queries import PuzzleSolutionNode
from ..models import Player, PuzzleSolution
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
    def mutate_and_get_row(cls, root, info, **input):
        """
        Execute the mutation and return the mutated row. By default this will
        instantiate the mutation's serializer class and save the serializer.
        The row should be of a model type that can map to this mutation's
        returned node type. E.g. if this mutation returns PuzzleSolutionNode,
        this method should return a PuzzleSolution.
        """

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
            return serializer.save()
        else:
            raise GraphQLError(
                "Input validation error(s)", extensions=serializer.errors
            )

    @classmethod
    def mutate_and_get_payload(cls, root, info, **input):
        node = cls.mutate_and_get_row(root, info, **input)
        edge = cls._meta.edge_class(node=node, cursor=offset_to_cursor(0))
        # Populate two output fields: one of just the node, one of the
        # node wrapped in an edge.
        return cls(**{cls._meta.node_field: node, cls._meta.edge_field: edge})


# TODO de-dupe some more of the logic between these. We should probably have
# a simple way to do CUD operations for any node type (and potentially make it
# easier to do extensions of that, like copy).


class SavePuzzleSolutionMutation(NodeMutation):
    """
    Create OR update a puzzle solution. Solutions are unique by their
    (name,player,puzzle) combo, so if a solution does not exist for that combo,
    a new one is created. If it does exist, the existing record is updated.

    The create and update operations are combined here to make it easy for a
    client to save a solution without having to know whether one exists for
    that player+puzzle+name yet. Think of this as a file save, where it will
    typically create the file if it doesn't existing before saving contents.

    We may want to split this into two mutations if that ends up being the
    pattern we use elsewhere, but for now it seems like it will be easier, at
    least for a CLI client.
    """

    class Meta:
        node_class = PuzzleSolutionNode
        serializer_class = PuzzleSolutionSerializer

    class Input:
        puzzle_id = graphene.ID(required=True)
        name = graphene.String(required=True)
        source_code = graphene.String(required=True)


class CopyPuzzleSolutionMutation(NodeMutation):
    """
    Copy a PuzzleSolution by ID. Returns the new puzzle solution. If provided,
    this will use the given name as the name for the new copy. If not, a new
    name will be generated.
    """

    class Meta:
        node_class = PuzzleSolutionNode

    class Input:
        id = graphene.ID(required=True)
        name = graphene.String()

    @classmethod
    def mutate_and_get_row(cls, root, info, **input):
        type, id = from_global_id(input["id"])
        puzzle_solution = PuzzleSolution.objects.get(id=id)
        puzzle_solution.id = None

        # We need to pick a new name for the copy. If the user gave us a name
        # to try, just use that. If it's already taken, we want to fail and
        # report that to the user. If they _didn't_ give us a name, then find
        # a new one. This option should _never fail_, so we may have to try
        # multiple times.

        if "name" in input:
            puzzle_solution.name = input["name"]
            # If this fails because of non-uniquness, we'll just raise an error
            # and return to the user
            puzzle_solution.save()
        else:
            # Keep incrementing the counter til we get a new name
            base_name = puzzle_solution.name
            counter = 1

            while True:
                puzzle_solution.name = f"{base_name} {counter}"
                try:
                    puzzle_solution.save()
                    break
                except IntegrityError:
                    counter += 1

        return puzzle_solution


class DeletePuzzleSolutionMutation(NodeMutation):
    """
    Delete a PuzzleSolution by ID. Returns the deleted node (if it existed).
    """

    class Meta:
        node_class = PuzzleSolutionNode

    class Input:
        id = graphene.ID(required=True)

    @classmethod
    def mutate_and_get_row(cls, root, info, **input):
        type, id = from_global_id(input["id"])
        puzzle_solution = PuzzleSolution.objects.get(id=id)
        puzzle_solution.delete()
        return puzzle_solution


class Mutation(graphene.ObjectType):
    save_puzzle_solution = SavePuzzleSolutionMutation.Field()
    copy_puzzle_solution = CopyPuzzleSolutionMutation.Field()
    delete_puzzle_solution = DeletePuzzleSolutionMutation.Field()
