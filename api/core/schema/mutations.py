import graphene
from graphene_django.rest_framework.mutation import SerializerMutation
from graphql_relay.connection.arrayconnection import offset_to_cursor

from .queries import HardwareSpecNode
from ..serializers import HardwareSpecSerializer

HardwareSpecNodeEdge = HardwareSpecNode._meta.connection.Edge


class ModifyHardwareSpecMutation(SerializerMutation):
    """
    A mutation to create OR update a single hardware spec. If the `id` field
    is included, an existing hardware spec will be updated. If not, a new one
    will be created.
    """

    class Meta:
        serializer_class = HardwareSpecSerializer
        model_operations = ["create", "update"]
        lookup_field = "id"

    class Input:
        name = graphene.String(required=True)
        num_registers = graphene.Int(required=True)
        num_stacks = graphene.Int(required=True)
        max_stack_length = graphene.Int(required=True)

    hardware_spec = graphene.Field(HardwareSpecNode)
    # Return an edge too to make it easy for Relay to add this to connections
    # https://github.com/graphql-python/graphene/issues/59#issuecomment-339648802
    hardware_spec_edge = graphene.Field(HardwareSpecNodeEdge)

    @classmethod
    def perform_mutate(cls, serializer, info):
        hardware_spec = serializer.save()
        edge = HardwareSpecNodeEdge(
            node=hardware_spec, cursor=offset_to_cursor(0)
        )
        return cls(
            hardware_spec=hardware_spec, hardware_spec_edge=edge, errors=None
        )


class Mutation(graphene.ObjectType):
    modify_hardware_spec = ModifyHardwareSpecMutation.Field()
