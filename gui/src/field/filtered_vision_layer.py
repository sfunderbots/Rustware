import pyqtgraph as pg
import queue
from collections import deque
import PyQt6
from PyQt6.QtCore import QPointF, QRectF
from PyQt6.QtGui import QFont, QPainterPath, QTransform
from src.visualizer.field.field_layer import FieldLayer
import src.visualizer.colors as colors
from src.world.world import World
from src.visualizer.util import create_text_path
from src.constants import BALL_MAX_RADIUS_METERS, ROBOT_MAX_RADIUS_METERS
from src.util.logger import LOG
from collections import namedtuple

# Make the ball a little easier to see
BALL_MAX_RADIUS_METERS *= 1.5


class FilteredVisionLayer(FieldLayer):
    TextSymbol = namedtuple("TextSymbol", "label symbol scale")

    def __init__(self):
        FieldLayer.__init__(self)
        self.buffer = deque(maxlen=1)
        self.cached_world = None
        self.is_friendly_team_blue = None

    def update_world(self, world: World):
        self.buffer.append(world)

    def update_friendly_color(self, is_friendly_team_blue: bool):
        self.is_friendly_team_blue = is_friendly_team_blue

    def draw_field(self, painter, field):
        painter.setPen(pg.mkPen(colors.FIELD_BOUNDARY_LINES))
        painter.drawRect(
            # Outer boundary
            QRectF(
                -field.field_x_length / 2.0 - field.boundary_buffer_size,
                -field.field_y_length / 2.0 - field.boundary_buffer_size,
                field.field_x_length + 2 * field.boundary_buffer_size,
                field.field_y_length + 2 * field.boundary_buffer_size,
            )
        )

        painter.setPen(pg.mkPen("w"))
        painter.drawRects(
            [
                # Touch lines
                QRectF(
                    -field.field_x_length / 2.0,
                    -field.field_y_length / 2.0,
                    field.field_x_length,
                    field.field_y_length,
                ),
                # defense 1
                QRectF(
                    -field.field_x_length / 2.0,
                    -field.defense_y_length / 2.0,
                    field.defense_x_length,
                    field.defense_y_length,
                ),
                # defense 2
                QRectF(
                    field.field_x_length / 2.0,
                    -field.defense_y_length / 2.0,
                    -field.defense_x_length,
                    field.defense_y_length,
                ),
                # goal 1
                QRectF(
                    -field.field_x_length / 2.0,
                    -field.goal_y_length / 2.0,
                    -field.goal_x_length,
                    field.goal_y_length,
                ),
                # goal 2
                QRectF(
                    field.field_x_length / 2.0,
                    -field.goal_y_length / 2.0,
                    field.goal_x_length,
                    field.goal_y_length,
                ),
            ]
        )
        # The halfway line
        painter.drawLine(
            QPointF(0, -field.field_y_length / 2.0),
            QPointF(0, field.field_y_length / 2.0),
        )

        painter.drawEllipse(
            QPointF(0.0, 0.0),
            field.center_circle_radius,
            field.center_circle_radius,
        )

    def draw_ball(self, painter, ball):
        painter.setPen(pg.mkPen(colors.BALL_COLOR))
        painter.setBrush(pg.mkBrush(colors.BALL_COLOR))
        painter.drawEllipse(
            QPointF(ball.position.x, ball.position.y),
            BALL_MAX_RADIUS_METERS,
            BALL_MAX_RADIUS_METERS,
        )

    def draw_robots(self, painter, robots: dict, color):
        painter.setPen(pg.mkPen(colors.BLACK))
        painter.setBrush(pg.mkBrush(color))
        convert_degree = -16
        for r in robots.values():
            rect = pg.QtCore.QRectF(
                r.position.x - ROBOT_MAX_RADIUS_METERS,
                r.position.y + ROBOT_MAX_RADIUS_METERS,
                ROBOT_MAX_RADIUS_METERS * 2,
                -ROBOT_MAX_RADIUS_METERS * 2,
            )
            painter.drawChord(
                rect,
                (r.orientation.degrees() + 45) * convert_degree,
                (270 * convert_degree),
            )

        painter.setPen(pg.mkPen(color=colors.BLACK, cosmetic=True))
        painter.setBrush(pg.mkBrush(colors.WHITE))
        for r in robots.values():
            painter.drawPath(
                create_text_path(
                    text=str(r.id),
                    pos=QPointF(r.position.x, r.position.y),
                    width=ROBOT_MAX_RADIUS_METERS * 1.25,
                    bold=True,
                )
            )

    def paint(self, painter, option, widget):
        """Paint this layer
        :param painter: The painter object to draw with
        :param option: Style information (unused)
        :param widget: The widget that we are painting on
        """
        try:
            self.cached_world = self.buffer.popleft()
        except IndexError:
            pass

        if self.cached_world and self.is_friendly_team_blue is not None:
            if self.cached_world.field:
                self.draw_field(painter, self.cached_world.field)
            self.draw_robots(
                painter,
                self.cached_world.enemy_robots,
                colors.YELLOW_ROBOT_COLOR
                if self.is_friendly_team_blue
                else colors.BLUE_ROBOT_COLOR,
            )
            self.draw_robots(
                painter,
                self.cached_world.friendly_robots,
                colors.BLUE_ROBOT_COLOR
                if self.is_friendly_team_blue
                else colors.YELLOW_ROBOT_COLOR,
            )
            if self.cached_world.ball:
                self.draw_ball(painter, self.cached_world.ball)
