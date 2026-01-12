from anise._anise import instrument

__all__ = []

for el in dir(instrument):
    if el and el[0] != "_":
        locals()[el] = getattr(instrument, el)
        __all__ += [el]
