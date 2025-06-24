from anise._anise import astro

for el in dir(astro):
    if el and el[0] != "_":
        locals()[el] = getattr(astro, el)