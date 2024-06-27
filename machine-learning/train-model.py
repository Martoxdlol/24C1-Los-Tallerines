from keras_preprocessing import image
from keras_preprocessing.image import ImageDataGenerator
from lib import train_generator, validation_generator
from model import model

model.summary()

model.compile(loss = 'categorical_crossentropy', optimizer='rmsprop', metrics=['accuracy'])

history = model.fit(train_generator, epochs=25, steps_per_epoch=20, validation_data = validation_generator, verbose = 1, validation_steps=3)

model.save("incidentes.keras")
