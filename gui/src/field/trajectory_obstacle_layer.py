import pyqtgraph as pg
import queue
from collections import deque
from PyQt6.QtCore import QPointF, QRectF
from src.visualizer.field.field_layer import FieldLayer
from src.geom.polygon import Polygon
import src.visualizer.colors as colors
from src.world.world import World
from src.constants import BALL_MAX_RADIUS_METERS, ROBOT_MAX_RADIUS_METERS
from src.motion.types.trajectory import Trajectory, TeamTrajectories
from src.util.logger import LOG


class TrajectoryObstacleLayer(FieldLayer):
    def __init__(self):
        FieldLayer.__init__(self)
        self.buffer = deque(maxlen=1)
        self.cached_obstacles = None

    def update_obstacles(self, robot_trajectory_obstacles):
        self.buffer.append(robot_trajectory_obstacles)

    def draw_obstacle(self, painter, obstacle: Polygon):
        painter.setPen(pg.mkPen("y"))

        for v0, v1 in zip(
            obstacle.vertices(), obstacle.vertices()[1:] + obstacle.vertices()[:1]
        ):
            painter.drawLine(
                QPointF(v0.x, v0.y),
                QPointF(v1.x, v1.y),
            )

    def draw_obstacles(self, painter, robot_trajectory_obstacles):
        for id, obstacles in robot_trajectory_obstacles.items():
            for o in obstacles:
                self.draw_obstacle(painter, o)

    def paint(self, painter, option, widget):
        """Paint this layer
        :param painter: The painter object to draw with
        :param option: Style information (unused)
        :param widget: The widget that we are painting on
        """
        try:
            self.cached_obstacles = self.buffer.popleft()
        except IndexError:
            pass

        if self.cached_obstacles is not None:
            self.draw_obstacles(painter, self.cached_obstacles)
