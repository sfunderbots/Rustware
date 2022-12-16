import pyqtgraph as pg
import queue
from collections import deque
from PyQt6.QtCore import QPointF, QRectF
from src.visualizer.field.field_layer import FieldLayer
import src.visualizer.colors as colors
from src.world.world import World
from src.constants import BALL_MAX_RADIUS_METERS, ROBOT_MAX_RADIUS_METERS
from src.motion.types.trajectory import Trajectory, TeamTrajectories
from src.util.logger import LOG


class TrajectoryLayer(FieldLayer):
    def __init__(self):
        FieldLayer.__init__(self)
        self.buffer = deque(maxlen=1)
        self.cached_trajectories = None

    def update_trajectories(self, trajectories: TeamTrajectories):
        self.buffer.append(trajectories)

    def draw_trajectory(self, painter, trajectory: Trajectory):
        painter.setPen(pg.mkPen("r"))

        for state0, state1 in zip(trajectory.states[:-1], trajectory.states[1:]):
            painter.drawLine(
                QPointF(state0.position.x, state0.position.y),
                QPointF(state1.position.x, state1.position.y),
            )

    def draw_trajectories(self, painter, trajectories: TeamTrajectories):
        for _, trajectory in trajectories.trajectories.items():
            self.draw_trajectory(painter, trajectory)

    def paint(self, painter, option, widget):
        """Paint this layer
        :param painter: The painter object to draw with
        :param option: Style information (unused)
        :param widget: The widget that we are painting on
        """
        try:
            self.cached_trajectories = self.buffer.popleft()
        except IndexError:
            pass

        if self.cached_trajectories is not None:
            self.draw_trajectories(painter, self.cached_trajectories)
