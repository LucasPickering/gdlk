from django.db import models
from django.contrib.postgres.functions import RandomUUID


class UUIDField(models.UUIDField):
    """
    A UUID field that serves as the primary key on a table. Minimal wrapper
    around Django's UUIDField.
    """

    def __init__(self, *args, **kwargs):
        super().__init__(
            *args,
            primary_key=True,
            editable=False,
            # Use the postgres function from pgcrypto for generating UUIDs
            default=RandomUUID
        )
