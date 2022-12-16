import os

from sys import path
from pathlib import Path

# So we can import protos from third_party
project_root = Path(__file__).parent.parent.parent
path.append(str(project_root))

import pyqtgraph as pg
from pyqtgraph.dockarea import *
from PyQt6.QtWidgets import QTabWidget, QMainWindow
from PyQt6.QtCore import QTimer
from PyQt6 import QtCore
from field.field import Field
# from gui.util.zmq_pub_sub import ZmqPubSub
from util.zmq_pub_sub import ZmqPubSub
from field.raw_vision_layer import RawVisionLayer
# from field.sim_control_layer import SimControlLayer
# from field.filtered_vision_layer import FilteredVisionLayer
# from field.trajectory_layer import TrajectoryLayer
# from field.trajectory_obstacle_layer import TrajectoryObstacleLayer
# from play.playinfo import PlayInfoWidget, MiscInfoWidget
# from third_party.ssl_vision.messages_robocup_ssl_wrapper_pb2 import SSL_WrapperPacket
# from proto import ssl_vision
# import proto_paths
from third_party.ssl_vision.messages_robocup_ssl_wrapper_pb2 import SSL_WrapperPacket
# from third_party.ssl_vision.

DIV_B_TOTAL_FIELD_X_LENGTH = 9
DIV_B_TOTAL_FIELD_Y_LENGTH = 6

class RustwareGui(QMainWindow):
    def __init__(self):
        super().__init__()

        self.pub_sub_manager = ZmqPubSub()
        # def foo(x):
        #     print("in callback")
        # self.pub_sub_manager.register_callback(foo, "test", SSL_WrapperPacket)

        self.setWindowTitle("Underbots GUI")

        self.refresh_timers = []

        self.tabs = QTabWidget()
        self.dock_area = DockArea()
        self._setup_gameplay_controls_dock_area()

        self.tabs.addTab(self.dock_area, "Gameplay Controls")

        self.setCentralWidget(self.tabs)

        self.pub_sub_manager.handle_callbacks_asynchronously()

    def _setup_gameplay_controls_dock_area(self):
        field_dock = Dock("field")
        field_dock.addWidget(
            self.setup_field_widget()
        )
        self.dock_area.addDock(field_dock)

        # logs_dock = Dock("Logs")
        # logs_dock.addWidget(self.setup_log_widget())
        # self.dock_area.addDock(logs_dock, "left", field_dock)
        #
        # performance_dock = Dock("Performance")
        # performance_dock.addWidget(self.setup_performance_plot().win)
        # self.dock_area.addDock(performance_dock, "bottom", field_dock)
        #
        # playinfo_dock = Dock("Play Info")
        # playinfo_dock.addWidget(self.setup_play_info_widget())
        # self.dock_area.addDock(playinfo_dock, "left", performance_dock)
        #
        # miscinfo_dock = Dock("Misc Info")
        # miscinfo_dock.addWidget(self.setup_misc_info_widget())
        # self.dock_area.addDock(miscinfo_dock, "bottom", playinfo_dock)

    def register_refresh_function(self, refresh_func, refresh_interval_ms=5):
        refresh_timer = QTimer()
        refresh_timer.setTimerType(QtCore.Qt.TimerType.PreciseTimer)
        refresh_timer.timeout.connect(refresh_func)
        refresh_timer.start(refresh_interval_ms)

        self.refresh_timers.append(refresh_timer)

    def setup_field_widget(self):
        field = Field(
            max_x_range=DIV_B_TOTAL_FIELD_X_LENGTH,
            max_y_range=DIV_B_TOTAL_FIELD_Y_LENGTH,
        )


        raw_vision_layer = RawVisionLayer()

        # def raw_vision_callback(x):
        #     pass
        #     data = SSL_WrapperPacket()
        #     data.ParseFromString(x)
        #     raw_vision_layer.update_detection_map(data)

        self.pub_sub_manager.register_callback(raw_vision_layer.update_detection_map, "test", SSL_WrapperPacket)
        # self.register_callback(
        #     lambda x: raw_vision_layer.update_friendly_defending_positive_side(
        #         x.friendly_team_info.defending_positive_side
        #     ),
        #     topic="game_data",
        # )
        field.add_layer("Raw Vision", raw_vision_layer)

        # return field
    #
    #     filtered_vision_layer = FilteredVisionLayer()
    #     # self.register_callback(
    #     #     callback=filtered_vision_layer.update_world, topic="world"
    #     # )
    #     # self.register_callback(
    #     #     callback=lambda x: filtered_vision_layer.update_friendly_color(
    #     #         x.friendly_team_info.is_blue
    #     #     ),
    #     #     topic="game_data",
    #     # )
    #     field.add_layer("Filtered Vision", filtered_vision_layer)
    #
    #     # sim_control_layer = SimControlLayer(
    #     #     pub_sim_command=lambda x: self.pub(
    #     #         obj=x, topic="sim_control", keep_only_last_message=False
    #     #     )
    #     # )
    #     # field.add_layer("Sim Control", sim_control_layer)
    #
    #     trajectory_layer = TrajectoryLayer()
    #     # self.register_callback(
    #     #     callback=trajectory_layer.update_trajectories, topic="trajectories"
    #     # )
    #     field.add_layer("Robot Trajectories", trajectory_layer)
    #
    #     trajectory_obstacle_layer = TrajectoryObstacleLayer()
    #     # self.register_callback(
    #     #     callback=lambda x: trajectory_obstacle_layer.update_obstacles(
    #     #         x.robot_trajectory_obstacles
    #     #     ),
    #     #     topic="gameplay_data",
    #     # )
    #     field.add_layer(
    #         "Trajectory Obstacles", trajectory_obstacle_layer, visible=False
    #     )
    #
        self.register_refresh_function(field.refresh, refresh_interval_ms=1)

        return field

def main():
    app = pg.mkQApp("Gui")
    w = RustwareGui()
    print(os.environ.values())
    w.show()
    app.exec()

if __name__ == "__main__":
    main()