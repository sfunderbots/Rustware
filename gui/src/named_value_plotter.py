import collections
import logging
import random
import time
from collections import deque
from queue import Queue
from src.util.logger import LOG
import distinctipy
from typing import List, Tuple, Dict

import pyqtgraph as pg
from pyqtgraph.Qt import QtGui

TIME_WINDOW_TO_DISPLAY_S = 20


class NamedValuePlotter(object):
    """Plot named values in real time with a scrolling plot"""

    def __init__(self, buffer_size=1000):
        """Initializes NamedValuePlotter.
        :param buffer_size: The size of the buffer to use for plotting.
        """
        self.win = pg.plot()
        self.win.disableAutoRange(axis="x")
        self.plots = {}
        self.data_x = {}
        self.data_y = {}
        self.legend = pg.LegendItem(
            (80, 60), offset=(70, 20), brush=pg.mkBrush("black")
        )
        self.legend.setParentItem(self.win.graphicsItem())
        self.time = time.time()
        # self.named_value_buffer = Queue()
        self.data: deque[Tuple[float, Dict[str, float]]] = deque()
        self.colors = distinctipy.get_colors(10)
        self.color_idx = 0

    def update_data(self, new_data: dict):
        self.data.append((time.time(), new_data))

    def refresh(self):
        """Refreshes NamedValuePlotter and updates data in the respective
        plots.
        """
        while self.data:
            timestamp, data = self.data.popleft()
            for name, value in data.items():
                # If named_value is new, create a plot and for the new value and
                # add it to necessary maps
                if name not in self.plots:
                    if self.color_idx == len(self.colors):
                        LOG.error(
                            "Generated {} unique colors but attempting to plot more than {} series".format(
                                len(self.colors), len(self.colors)
                            )
                        )
                        self.color_idx -= 1
                    r, g, b = self.colors[self.color_idx]
                    self.color_idx += 1
                    self.plots[name] = self.win.plot(
                        pen=QtGui.QColor(r * 255, g * 255, b * 255),
                        name=name,
                        disableAutoRange=True,
                        brush=None,
                    )

                    self.plots[name].setDownsampling(method="peak")
                    self.data_x[name] = deque()
                    self.data_y[name] = deque()
                    self.legend.addItem(self.plots[name], name)

                # Add incoming data to existing deques of data
                self.data_x[name].append(timestamp - self.time)
                self.data_y[name].append(value)
        # Discard data outside the time window
        for name in self.data_x:
            while self.data_x[name]:
                x = self.data_x[name][0]
                if x < time.time() - self.time - TIME_WINDOW_TO_DISPLAY_S:
                    self.data_x[name].popleft()
                    self.data_y[name].popleft()
                else:
                    break

        for named_value, plot in self.plots.items():
            plot.setData(self.data_x[named_value], self.data_y[named_value])

        self.win.setRange(
            xRange=[
                time.time() - self.time - TIME_WINDOW_TO_DISPLAY_S,
                time.time() - self.time,
            ]
        )
