import graphene

import core.schema


# Inherit the Query class from each app
class Query(core.schema.Query, graphene.ObjectType):
    pass


schema = graphene.Schema(query=Query)
