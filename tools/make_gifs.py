"""Arma los GIFs de la carpeta media/ a partir de los cuadros que genera
`cargo run --release -- --save-frames ...` (ver README).

Uso: python tools/make_gifs.py
"""
import glob
import os
import sys

from PIL import Image

# carpeta de cuadros -> (nombre del gif, ms por cuadro)
GIFS = {
    "estrella": ("estrella.gif", 80),
    "planeta_rocoso": ("planeta_rocoso_luz.gif", 80),
    "gigante_gaseoso": ("gigante_gaseoso.gif", 80),
    "planeta_de_lava": ("planeta_de_lava.gif", 80),
}
SIZE = (480, 360)

os.makedirs("media", exist_ok=True)
for folder, (name, ms) in GIFS.items():
    files = sorted(glob.glob(f"frames/{folder}/frame_*.png"))
    if not files:
        print(f"Sin cuadros en frames/{folder}, se omite", file=sys.stderr)
        continue
    frames = [Image.open(f).convert("RGB").resize(SIZE, Image.LANCZOS) for f in files]
    frames[0].save(f"media/{name}", save_all=True, append_images=frames[1:], duration=ms, loop=0, optimize=True)
    print(f"media/{name}: {len(frames)} cuadros, {os.path.getsize(f'media/{name}') // 1024} KB")
