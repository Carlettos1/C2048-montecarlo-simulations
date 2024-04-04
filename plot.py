import numpy as np
import matplotlib.pyplot as plt

i = np.loadtxt("./log_index", dtype="int")
e = np.loadtxt(f"./energy_avg_{i}.log")[1:]
v = np.loadtxt(f"./victories_{i}.log")[1:]

fig, axes = plt.subplot_mosaic("e\nv", figsize=(20, 9))
axes["e"].plot(e)
axes["e"].set_title("Energies vs T")
axes["v"].plot(v)
axes["v"].set_title("Victories vs T")

plt.show()