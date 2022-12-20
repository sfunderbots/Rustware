import pyqtgraph as pg
from PyQt6.QtCore import Qt
from src.field.field_layer import FieldLayer
import src.colors as colors
from PyQt6.QtCore import QPointF, QLineF


class SimControlLayer(FieldLayer):
    """
    A layer that implements live simulator controls. A bit of a hack, but this makes
    it easy to toggle on/off for now
    """

    def __init__(self, pub_sim_command):
        FieldLayer.__init__(self)
        self._pub_sim_command = pub_sim_command

        self._last_mouse_point: QPointF = None
        self._ball_command_line: QLineF = None

    def keyPressEvent(self, event):
        if (
            event.key() == Qt.Key.Key_B
            and not event.isAutoRepeat()
            and self._last_mouse_point is not None
        ):
            self._ball_command_line = QLineF(
                self._last_mouse_point, self._last_mouse_point
            )

    def keyReleaseEvent(self, event):
        if (
            event.key() == Qt.Key.Key_B
            and not event.isAutoRepeat()
            and self._ball_command_line is not None
        ):
            # command = SimControlCommand(
            #     ball=Ball(
            #         position=toPoint(self._ball_command_line.p1()),
            #         velocity=toVector(self._ball_command_line),
            #     )
            # )
            # TODO: For some reason the key has to be pressed twice before commands start getting through, but
            # it works as expected after that. Not sure why, but should check zmq isn't always
            # one tick behind
            # self._pub_sim_command(command)
            self._ball_command_line = None

    def onMouseMoved(self, pos: QPointF):
        self._last_mouse_point = pos

        if self._ball_command_line is not None:
            self._ball_command_line.setP2(pos)

    def draw_ball_set_velocity(self, painter):
        if self._ball_command_line is None:
            return
        painter.setPen(pg.mkPen(colors.BALL_COLOR))
        painter.drawLine(self._ball_command_line)

    def paint(self, painter, option, widget):
        self.draw_ball_set_velocity(painter)
