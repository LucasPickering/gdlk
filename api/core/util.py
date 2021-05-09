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


def pop_many(d, keys):
    """
    Pop multiple keys from a dictionary, and put all the popped entries into
    their own dictionary. Returns the dictionary of popped values, which will
    have the same length as the input list of keys (assuming no duplicate keys,
    and all keys are present in the input dictionary.)
    """
    rv = {}
    for key in keys:
        rv[key] = d.pop(key)
    return rv
