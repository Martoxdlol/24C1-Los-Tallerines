from keras.models import load_model
from keras.preprocessing import image
import numpy as np
import sys
from lib import INDICES

# Filename from argv
filename = sys.argv[1] 

model = load_model('incidentes.keras')

img = image.load_img(filename, target_size=(150, 150))
x = image.img_to_array(img)
x = np.expand_dims(x, axis=0)

images = np.vstack([x])
classes = model.predict(images, batch_size=10)

print(classes)

for key in INDICES:
    i = INDICES[key]
    print(f'{key}: {classes[0][i]}')
