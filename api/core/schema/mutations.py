from enum import Enum
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

    operation = None
    serializer_class = None
    node_class = None
    edge_class = None
    node_field = None
    edge_field = None


class NodeMutationOperation(Enum):
    """
    The different type of pre-defined operations that a node mutation can
    perform. Use CUSTOM to write your own mutation logic.
    """

    CREATE = "create"
    UPDATE = "update"
    DELETE = "delete"
    CUSTOM = "custom"


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
    - `operation` (must be an option from NodeMutationOperation)
    - `node_class`
    - `serializer_class` (only required for CREATE and UPDATE operations)

    ### Optional `Meta` Fields
    - `node_field`
    - `edge_field`

    If not provided, the optional fields will be derived automatically
    """

    @classmethod
    def __init_subclass_with_meta__(
        cls,
        operation=None,
        serializer_class=None,
        node_class=None,
        _meta=None,
        **options,
    ):
        if not _meta:
            _meta = NodeMutationOptions(cls)

        if operation not in set(NodeMutationOperation):
            raise Exception(
                f"Invalid node mutation operation: expected one of "
                f"{list(NodeMutationOperation)}, got {operation}"
            )

        _meta.operation = operation
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
        serializer_class = cls._meta.serializer_class
        data = {
            # Always pass in the player ID from the request, the serializer
            # may or may not use it. The subclass is free to override this
            # value by defining its own player_id input field.
            "player_id": player.id,
            **input,
        }
        if cls._meta.operation == NodeMutationOperation.CREATE:
            serializer = serializer_class(data=data)
        elif cls._meta.operation == NodeMutationOperation.UPDATE:
            # TODO make this a bit more dynamic or cleaner or something
            # IDK it feels jank but not sure what the right fix is
            serializer = serializer_class(
                serializer_class.Meta.model.objects.get(
                    id=from_global_id(input["id"])[1]
                ),
                partial=True,
                data=data,
            )
        # TODO implement DELETE and CUSTOM operations

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


class CreatePuzzleSolutionMutation(NodeMutation):
    """
    Create a puzzle solution. Solutions are unique by their
    (name,player,puzzle), so if that combo already exists, this will fail.
    """

    class Meta:
        operation = NodeMutationOperation.CREATE
        node_class = PuzzleSolutionNode
        serializer_class = PuzzleSolutionSerializer

    class Input:
        puzzle_id = graphene.ID(required=True)
        name = graphene.String(required=True)
        source_code = graphene.String(required=True)


class UpdatePuzzleSolutionMutation(NodeMutation):
    """
    Update a puzzle solution by ID. This is a partial update, so any field
    that's provided will be modified, and any other field will remain untouched.
    """

    class Meta:
        operation = NodeMutationOperation.UPDATE
        node_class = PuzzleSolutionNode
        serializer_class = PuzzleSolutionSerializer

    class Input:
        id = graphene.ID(required=True)
        puzzle_id = graphene.String()
        name = graphene.String()
        source_code = graphene.String()


class CopyPuzzleSolutionMutation(NodeMutation):
    """
    Copy a PuzzleSolution by ID. Returns the new puzzle solution. If provided,
    this will use the given name as the name for the new copy. If not, a new
    name will be generated.
    """

    class Meta:
        operation = NodeMutationOperation.CUSTOM
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
        operation = NodeMutationOperation.DELETE
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
    create_puzzle_solution = CreatePuzzleSolutionMutation.Field()
    update_puzzle_solution = UpdatePuzzleSolutionMutation.Field()
    copy_puzzle_solution = CopyPuzzleSolutionMutation.Field()
    delete_puzzle_solution = DeletePuzzleSolutionMutation.Field()
