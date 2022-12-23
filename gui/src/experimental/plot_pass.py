import json

from src.constants import ROBOT_MAX_RADIUS_METERS
import matplotlib.pyplot as plt


def plot_score_function(data: dict):
    fig, ax = plt.subplots()

    z = data["z"]
    x = data["x"]
    y = data["y"]

    c = ax.pcolormesh(x, y, z, cmap="RdBu", vmin=0, vmax=1)
    ax.set_title(
        "Pass Score (higher is better) - Speed: {}, Time: {}".format(
            data["speed"], data["time_offset"]
        )
    )
    fig.colorbar(c, ax=ax)

    pass_start_circle = plt.Circle(
        (data["start"][0], data["start"][1]), 0.05, color="y"
    )
    ax.add_patch(pass_start_circle)

    for x, y, vx, vy in [tuple(r) for r in data["friendly_robots"]]:
        f_circle = plt.Circle((x, y), ROBOT_MAX_RADIUS_METERS, color="g")
        ax.add_patch(f_circle)
    for x, y, vx, vy in [tuple(r) for r in data["enemy_robots"]]:
        f_circle = plt.Circle((x, y), ROBOT_MAX_RADIUS_METERS, color="k")
        ax.add_patch(f_circle)

    plt.show()


def main():
    with open("/tmp/underbots_passing_plot_data.json", "r") as infile:
        data = json.load(infile)
        plot_score_function(data)


if __name__ == "__main__":
    main()
