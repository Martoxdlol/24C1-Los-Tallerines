from keras.models import load_model
from keras.preprocessing import image
import numpy as np
import sys
from lib import INDICES
from os import scandir, getcwd
from os.path import abspath

def ls(ruta = getcwd()):
    return [abspath(arch.path) for arch in scandir(ruta) if arch.is_file()]



# Filename from argv
carpeta = sys.argv[1] 

model = load_model('incidentes.keras')
cant_incendios = 0
cant_no_incendios = 0
INDICE_INCENDIOS = INDICES['incendios']
INDICE_NO_INCENDIOS = INDICES['normal']
lista_arq = ls(carpeta)
for imagen in lista_arq:

    img = image.load_img(imagen, target_size=(150, 150))
    x = image.img_to_array(img)
    x = np.expand_dims(x, axis=0)

    images = np.vstack([x])
    classes = model.predict(images, batch_size=10)
    if classes[0][INDICE_INCENDIOS] > classes[0][INDICE_NO_INCENDIOS]:
        cant_incendios += 1
    else:
        cant_no_incendios += 1
        
print(f'Cantidad de incendios: {cant_incendios}')
print(f'Cantidad de no incendios: {cant_no_incendios}')

