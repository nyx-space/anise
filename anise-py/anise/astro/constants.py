from anise._anise import constants

for el in dir(constants):
    if el and el[0] != "_":
        locals()[el] = getattr(constants, el)