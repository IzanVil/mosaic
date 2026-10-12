"""Genera proyectos de prueba para medir el rendimiento.

Uso: generar.py CARPETA

Crea tres raíces, `a` (500 proyectos), `b` (1.500) y `c` (3.000), para medir
con 500, 2.000 y 5.000 escaneando una, dos o las tres. Cada proyecto tiene su
manifiesto y unos ficheros de código; el 70 % es un repositorio Git con un
commit, y de esos uno de cada cinco tiene cambios sin commitear.
"""

import os
import random
import subprocess
import sys

TIPOS = [
    ("Cargo.toml", "src", ".rs"),
    ("package.json", "src", ".ts"),
    ("pyproject.toml", "app", ".py"),
    ("go.mod", "cmd", ".go"),
    ("pom.xml", "src/main/java", ".java"),
]
GIT = ["git", "-c", "user.name=Mosaic", "-c", "user.email=mosaic@example.invalid", "-c", "init.defaultBranch=main"]

base = sys.argv[1]
random.seed(7)
n = 0
for raiz, cuantos in (("a", 500), ("b", 1500), ("c", 3000)):
    for i in range(cuantos):
        manifiesto, carpeta, ext = TIPOS[n % len(TIPOS)]
        # Agrupados como suelen estarlo: cliente/proyecto, a uno o dos niveles.
        grupo = f"grupo-{i % 40:02d}"
        ruta = os.path.join(base, raiz, grupo, f"proyecto-{n:05d}") if i % 3 else os.path.join(base, raiz, f"proyecto-{n:05d}")
        os.makedirs(os.path.join(ruta, carpeta), exist_ok=True)
        open(os.path.join(ruta, manifiesto), "w").write("[package]\n")
        open(os.path.join(ruta, "README.md"), "w").write(f"# proyecto-{n:05d}\n")
        for k in range(12):
            open(os.path.join(ruta, carpeta, f"m{k}{ext}"), "w").write("// código\n")
        if manifiesto == "package.json" and n % 10 == 0:
            # Un node_modules de muestra: el escáner tiene que saltarlo.
            os.makedirs(os.path.join(ruta, "node_modules", "dep", "lib"), exist_ok=True)
            for k in range(30):
                open(os.path.join(ruta, "node_modules", "dep", "lib", f"f{k}.js"), "w").write("x\n")
        if n % 10 < 7:
            subprocess.run(GIT + ["init", "-q", ruta], check=True)
            subprocess.run(GIT + ["-C", ruta, "add", "-A"], check=True)
            subprocess.run(GIT + ["-C", ruta, "commit", "-qm", "inicio"], check=True)
            if n % 5 == 0:
                open(os.path.join(ruta, "PENDIENTE.md"), "w").write("sin commitear\n")
        n += 1
print(n, "proyectos en", base)
