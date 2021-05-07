from graphene import relay, ObjectType
from graphene_django import DjangoObjectType, DjangoConnectionField

from .models import HardwareSpec, ProgramSpec


class HardwareSpecNode(DjangoObjectType):
    class Meta:
        model = HardwareSpec
        interfaces = (relay.Node,)


class ProgramSpecNode(DjangoObjectType):
    class Meta:
        model = ProgramSpec
        interfaces = (relay.Node,)


class Query(ObjectType):
    hardware_spec = relay.Node.Field(HardwareSpecNode)
    hardware_specs = DjangoConnectionField(HardwareSpecNode)
