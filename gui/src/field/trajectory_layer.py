import math

import pyqtgraph as pg
from PyQt6.QtCore import QPointF, QRectF
from .field_layer import FieldLayer
from src.constants import (
    METERS_PER_MILLIMETER,
    BALL_MAX_RADIUS_METERS,
    ROBOT_MAX_RADIUS_METERS,
)
import src.colors as colors
from threading import Lock
from proto.visualization_pb2 import Visualization

# Make the ball a little easier to see
BALL_MAX_RADIUS_METERS *= 1.5


class TrajectoryLayer(FieldLayer):
    def __init__(self):
        FieldLayer.__init__(self)
        self.trajectories = []
        self.lock = Lock()

    def update_trajectories(self, trajectories):
        with self.lock:
            self.trajectories = trajectories

    def draw_trajectories(self, painter, trajectories):
        painter.setPen(pg.mkPen("r"))

        for trajectory in trajectories:
            for p1, p2 in zip(trajectory.points[:-1], trajectory.points[1:]):
                painter.drawLine(
                    QPointF(
                        p1.x, p1.y
                    ),
                    QPointF(
                        p2.x, p2.y
                    )
                )

    def paint(self, painter, option, widget):
        """Paint this layer
        :param painter: The painter object to draw with
        :param option: Style information (unused)
        :param widget: The widget that we are painting on
        """
        with self.lock:
            self.draw_trajectories(painter, self.trajectories)
