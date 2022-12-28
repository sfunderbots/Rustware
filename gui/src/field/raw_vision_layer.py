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
from third_party.ssl_vision.messages_robocup_ssl_wrapper_pb2 import SSL_WrapperPacket, SSL_WrapperPackets

# Make the ball a little easier to see
BALL_MAX_RADIUS_METERS *= 1.5


class RawVisionLayer(FieldLayer):
    def __init__(self):
        FieldLayer.__init__(self)
        self.cached_geometry = None
        self.camera_id_map = dict()
        self.lock = Lock()
        self.friendly_defending_positive_side = None

    def update_detection_map(self, wrapper_packets: SSL_WrapperPackets):
        with self.lock:
            for packet in wrapper_packets.packets:
                if packet.HasField("detection"):
                    detection = packet.detection
                    camera_id = detection.camera_id
                    self.camera_id_map[camera_id] = detection
                if packet.HasField("geometry"):
                    self.cached_geometry = packet.geometry

    def update_friendly_defending_positive_side(self, friendly_defending_positive_side):
        with self.lock:
            self.friendly_defending_positive_side = friendly_defending_positive_side

    def draw_field(self, painter, geometry_data):
        if not geometry_data:
            return
        field = geometry_data.field
        painter.setPen(pg.mkPen("w"))

        for line in field.field_lines:
            painter.drawLine(
                QPointF(
                    line.p1.x * METERS_PER_MILLIMETER, line.p1.y * METERS_PER_MILLIMETER
                ),
                QPointF(
                    line.p2.x * METERS_PER_MILLIMETER, line.p2.y * METERS_PER_MILLIMETER
                ),
            )

        for arc in field.field_arcs:
            painter.drawEllipse(
                QPointF(
                    arc.center.x * METERS_PER_MILLIMETER,
                    arc.center.y * METERS_PER_MILLIMETER,
                ),
                arc.radius * METERS_PER_MILLIMETER,
                arc.radius * METERS_PER_MILLIMETER,
            )

        # The goals aren't included in the field lines so draw them separately
        painter.drawRects(
            [
                QRectF(
                    -field.field_length / 2.0 * METERS_PER_MILLIMETER,
                    -field.goal_width / 2.0 * METERS_PER_MILLIMETER,
                    -field.goal_depth * METERS_PER_MILLIMETER,
                    field.goal_width * METERS_PER_MILLIMETER,
                ),
                QRectF(
                    field.field_length / 2.0 * METERS_PER_MILLIMETER,
                    -field.goal_width / 2.0 * METERS_PER_MILLIMETER,
                    field.goal_depth * METERS_PER_MILLIMETER,
                    field.goal_width * METERS_PER_MILLIMETER,
                ),
            ]
        )

    def draw_balls(self, painter, balls):
        painter.setPen(pg.mkPen(colors.BALL_COLOR))
        painter.setBrush(pg.mkBrush(colors.BALL_COLOR))
        for b in balls:
            painter.drawEllipse(
                QPointF(b.x * METERS_PER_MILLIMETER, b.y * METERS_PER_MILLIMETER),
                BALL_MAX_RADIUS_METERS,
                BALL_MAX_RADIUS_METERS,
            )

    def draw_robots(self, painter, robots, color):
        painter.setPen(pg.mkPen(colors.BLACK))
        painter.setBrush(pg.mkBrush(color))
        convert_degree = -16
        for r in robots:
            painter.drawChord(
                pg.QtCore.QRectF(
                    r.x * METERS_PER_MILLIMETER - ROBOT_MAX_RADIUS_METERS,
                    r.y * METERS_PER_MILLIMETER + ROBOT_MAX_RADIUS_METERS,
                    ROBOT_MAX_RADIUS_METERS * 2,
                    -ROBOT_MAX_RADIUS_METERS * 2,
                ),
                (math.degrees(r.orientation) + 45) * convert_degree,
                (270 * convert_degree),
            )

    def paint(self, painter, option, widget):
        """Paint this layer
        :param painter: The painter object to draw with
        :param option: Style information (unused)
        :param widget: The widget that we are painting on
        """
        with self.lock:
            # if self.friendly_defending_positive_side is not None:
            #     if self.friendly_defending_positive_side:
            #         # Invert the painter so it still draws the friendly team on the left
            #         painter.scale(-1, -1)
            self.draw_field(painter, self.cached_geometry)
            for detection in self.camera_id_map.values():
                self.draw_robots(
                    painter, detection.robots_yellow, colors.YELLOW_ROBOT_COLOR
                )
                self.draw_robots(
                    painter, detection.robots_blue, colors.BLUE_ROBOT_COLOR
                )
                self.draw_balls(painter, detection.balls)
