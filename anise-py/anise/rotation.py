from anise._anise import rotation

for el in dir(rotation):
    if el and el[0] != "_":
        locals()[el] = getattr(rotation, el)