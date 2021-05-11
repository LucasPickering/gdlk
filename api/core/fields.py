import uuid
from django.db import models


class UUIDField(models.UUIDField):
    """
    A UUID field that serves as the primary key on a table. Minimal wrapper
    around Django's UUIDField. This field will be auto-populated during an
    insert.
    """

    def __init__(self, *args, **kwargs):
        super().__init__(
            *args,
            primary_key=True,
            editable=False,
            # Generate a UUID **in Python** before insert. This is easier than
            # generated the UUID in Postgres because:
            # 1. We don't need an extra migration to enable the pg extension
            # 2. Avoids django annoyances with loading the generated value
            #   after insert
            default=uuid.uuid4
        )
