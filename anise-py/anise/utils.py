from anise._anise import utils

for el in dir(utils):
    if el and el[0] != "_":
        locals()[el] = getattr(utils, el)