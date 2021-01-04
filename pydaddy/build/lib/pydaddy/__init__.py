BROADCAST_ADDR: int = 0xffff


def is_iter(obj):
    """
    Checks if an object behaves iterably.
    Args:
        obj (any): Entity to check for iterability.
    Returns:
        is_iterable (bool): If `obj` is iterable or not.
    Notes:
        Strings are *not* accepted as iterable (although they are
        actually iterable), since string iterations are usually not
        what we want to do with a string.
    """
    if isinstance(obj, (str, bytes)):
        return False

    try:
        return iter(obj) and True
    except TypeError:
        return False


def make_iter(obj):
    """
    Makes sure that the object is always iterable.
    Args:
        obj (any): Object to make iterable.
    Returns:
        iterable (list or iterable): The same object
            passed-through or made iterable.
    """
    return not is_iter(obj) and [obj] or obj