from keras.models import load_model
from keras.preprocessing import image
import numpy as np

model = load_model('incidentes.keras')

img = image.load_img('./dataset/test/normal/non_fire.43.png', target_size=(150, 150))
x = image.img_to_array(img)
x = np.expand_dims(x, axis=0)

images = np.vstack([x])
classes = model.predict(images, batch_size=10)

print(classes)