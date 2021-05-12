from rest_framework import serializers
from graphql_relay import from_global_id, to_global_id

from .schema.queries import PuzzleNode
from .models import PuzzleSolution


class NodeIdField(serializers.CharField):
    """
    A serializer field that represents a GraphQL node ID. Internally, the
    deserialized value will be a UUID that can be used with the database.

    This takes in the type of the node being reference, so that it can use the
    node type's name for serialization and deserialization.
    """

    def __init__(self, node_class, **kwargs):
        super().__init__(**kwargs)
        self._type_name = node_class.__name__

    def to_representation(self, value):
        return to_global_id(self._type_name, value)

    def to_internal_value(self, data):
        try:
            type, id = from_global_id(data)
        except Exception as e:
            raise serializers.ValidationError(f"Invalid node ID: {e}")

        # Make sure the ID was for the type we expected
        if type != self._type_name:
            raise serializers.ValidationError(
                f"Invalid node ID: Expected ID for type {self._type_name},"
                f" received ID for type {type}"
            )

        return id


class PuzzleSolutionSerializer(serializers.ModelSerializer):
    player_id = serializers.UUIDField()
    puzzle_id = NodeIdField(PuzzleNode)
    name = serializers.CharField(max_length=50)
    source_code = serializers.CharField(allow_blank=True, trim_whitespace=False)

    class Meta:
        model = PuzzleSolution
        fields = (
            "player_id",
            "puzzle_id",
            "name",
            "source_code",
        )
