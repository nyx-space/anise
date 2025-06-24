from anise._anise import time

for el in dir(time):
    if el and el[0] != "_":
        locals()[el] = getattr(time, el)