import sys
import rust_png_reader
import matplotlib.image as mpimg

from matplotlib import pyplot as plt

img1 = rust_png_reader.read_png(sys.argv[1])
img2 = mpimg.imread(sys.argv[1])

fig, (ax1, ax2) = plt.subplots(1, 2)

ax1.imshow(img1)
ax2.imshow(img2)

ax1.title.set_text("Rust png decoder")
ax2.title.set_text("Matplotlib png decoder")

plt.show()
