import itertools
import cProfile

import time
import numpy as np
from dataclasses import dataclass
from src.constants import ROBOT_MAX_RADIUS_METERS
import math
import matplotlib.pyplot as plt
from typing import Tuple


def plot_score_function(data: dict):
    fig, ax = plt.subplots()

    z = data["z"]
    x = data["x"]
    y = data["y"]

    c = ax.pcolormesh(x, y, z, cmap='RdBu', vmin=0, vmax=1)
    ax.set_title("Pass Score (higher is better) - Speed: {}, Time: {}".format(data["speed"], data["time_offset"]))
    fig.colorbar(c, ax=ax)

    pass_start_circle = plt.Circle((data["start"][0], data["start"][1]), 0.05, color='y')
    ax.add_patch(pass_start_circle)

    for x, y, vx, vy in [tuple(r) for r in data["friendly_robots"]]:
        f_circle = plt.Circle((x, y), ROBOT_MAX_RADIUS_METERS, color='g')
        ax.add_patch(f_circle)
    for x, y, vx, vy in [tuple(r) for r in data["enemy_robots"]]:
        f_circle = plt.Circle((x, y), ROBOT_MAX_RADIUS_METERS, color='k')
        ax.add_patch(f_circle)

    plt.show()

def main():
    data = {
        "speed": 5,
        "time_offset": 0.15,
        "start": [0, 1],
        "friendly_robots": [
            [1, 1, 0, 0],
            [-2, 0, 0, 0],
            [0, -1, 0, 0],
        ],
        "enemy_robots": [
            [-1, 1, 0, 0],
            [2, 0, 0, 0],
            [0, 1, 0, 0],
        ],
        "z": [
            [1, 0.8, 0.1, 0.2, 0.4],
            [0.8, 0.9, 0.1, 0.2, 0.7],
            [0.4, 0.3, 0.45, 0.8, 0.4],
        ],
        "x": [
            [-1, -0.5, 0, 0.5, 1],
            [-1, -0.5, 0, 0.5, 1],
            [-1, -0.5, 0, 0.5, 1],
        ],
        "y": [
            [1, 1, 1, 1, 1],
            [0, 0, 0, 0, 0],
            [-1, -1, -1, -1, -1],
        ]
    }
    plot_score_function(data)

if __name__ == '__main__':
    # main(plot=True)
    main()
    # originall 0.06-0.07 ms per call
    # After first np change, now 0.13 ms per call :(
    # Back to 0.06
    # cProfile.run("main()", sort="tottime")

    # start = time.time()
    # print(bb_time_to_position_1d(xi=0, vi=2, xf=4, vf=0, a=3, v_max=3))
    # print(bb_time_to_position_1d(xi=0, vi=-2, xf=4, vf=0, a=3, v_max=3))
    # print("Took {} ms".format((time.time()-start)*1000))

    # start = time.time()
    # print(bb_time_to_position())
    # print("Took {} ms".format((time.time()-start)*1000))
