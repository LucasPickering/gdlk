import re


def get_node_field_name(node_class):
    """
    Get a reasonable field name based on a node class. This will lop off the
    "Node" at the end (if present), then convert to snake case.
    """

    name = node_class.__name__

    # Remove "Node" suffix
    suffix = "Node"
    if name.endswith(suffix):
        name = name[: -len(suffix)]

    # Convert to snake case
    name = re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()

    return name
