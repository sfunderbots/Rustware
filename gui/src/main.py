from pathlib import Path

from config.config_pb2 import Config
from google.protobuf.text_format import MessageToString, Parse

import signal

import os
from field.sim_control_layer import SimControlLayer
import pyqtgraph as pg
from pyqtgraph.dockarea import *
from PyQt6.QtWidgets import QTabWidget, QMainWindow
from PyQt6.QtCore import QTimer
from PyQt6 import QtCore
from field.field import Field
from named_value_plotter import NamedValuePlotter
from log_widget import LogWidget
from play.playinfo import PlayInfoWidget, MiscInfoWidget

# from gui.util.zmq_pub_sub import ZmqPubSub
from util.zmq_pub_sub import ZmqPubSub
from field.raw_vision_layer import RawVisionLayer

# from field.sim_control_layer import SimControlLayer
from field.filtered_vision_layer import FilteredVisionLayer

# from field.trajectory_layer import TrajectoryLayer
# from field.trajectory_obstacle_layer import TrajectoryObstacleLayer
# from play.playinfo import PlayInfoWidget, MiscInfoWidget
# from third_party.ssl_vision.messages_robocup_ssl_wrapper_pb2 import SSL_WrapperPacket
# from proto import ssl_vision
# import proto_paths
from third_party.ssl_vision.messages_robocup_ssl_wrapper_pb2 import SSL_WrapperPacket
from proto.visualization_pb2 import Visualization
from proto.metrics_pb2 import NodePerformance

# from third_party.ssl_vision.

DIV_B_TOTAL_FIELD_X_LENGTH = 9
DIV_B_TOTAL_FIELD_Y_LENGTH = 6


class RustwareGui(QMainWindow):
    def __init__(self):
        super().__init__()

        self.config = load_config()
        self.pub_sub_manager = ZmqPubSub(self.config.gui_bridge.unix_socket_prefix)

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
        field_dock.addWidget(self.setup_field_widget())
        self.dock_area.addDock(field_dock)

        logs_dock = Dock("Logs")
        logs_dock.addWidget(self.setup_log_widget())
        self.dock_area.addDock(logs_dock, "left", field_dock)

        performance_dock = Dock("Performance")
        performance_dock.addWidget(self.setup_performance_plot().win)
        self.dock_area.addDock(performance_dock, "bottom", field_dock)

        playinfo_dock = Dock("Play Info")
        playinfo_dock.addWidget(self.setup_play_info_widget())
        self.dock_area.addDock(playinfo_dock, "left", performance_dock)

        miscinfo_dock = Dock("Misc Info")
        miscinfo_dock.addWidget(self.setup_misc_info_widget())
        self.dock_area.addDock(miscinfo_dock, "bottom", playinfo_dock)

    def register_refresh_function(self, refresh_func, refresh_interval_ms=5):
        refresh_timer = QTimer()
        refresh_timer.setTimerType(QtCore.Qt.TimerType.PreciseTimer)
        refresh_timer.timeout.connect(refresh_func)
        refresh_timer.start(refresh_interval_ms)

        self.refresh_timers.append(refresh_timer)

    def setup_play_info_widget(self):
        play_info = PlayInfoWidget()

        self.register_refresh_function(play_info.refresh)

        return play_info

    def setup_misc_info_widget(self):
        misc_info = MiscInfoWidget()

        self.register_refresh_function(misc_info.refresh)

        return misc_info

    def setup_log_widget(self):
        logs = LogWidget()
        self.register_refresh_function(logs.refresh)

        return logs

    def setup_field_widget(self):
        field = Field(
            max_x_range=DIV_B_TOTAL_FIELD_X_LENGTH,
            max_y_range=DIV_B_TOTAL_FIELD_Y_LENGTH,
        )

        raw_vision_layer = RawVisionLayer()
        self.pub_sub_manager.register_callback(
            raw_vision_layer.update_detection_map, self.config.gui_bridge.ssl_vision_topic, SSL_WrapperPacket
        )
        field.add_layer("Raw Vision", raw_vision_layer)

        def world_callback(msg: Visualization):
            if msg.HasField("world"):
                filtered_vision_layer.update_world(msg.world)

        filtered_vision_layer = FilteredVisionLayer()
        self.pub_sub_manager.register_callback(world_callback, self.config.gui_bridge.world_topic, Visualization)
        field.add_layer("Filtered Vision", filtered_vision_layer)

        sim_control_layer = SimControlLayer(
            # pub_sim_command=lambda x: self.pub(
            #     obj=x, topic="sim_control", keep_only_last_message=False
            # )
            pub_sim_command=lambda x: print("TODO: pub sim control")
        )
        field.add_layer("Sim Control", sim_control_layer)

        self.register_refresh_function(field.refresh, refresh_interval_ms=1)

        return field

    def setup_performance_plot(self):
        named_value_plotter = NamedValuePlotter()

        def callback(x):
            named_value_plotter.update_data(x.mean_publish_period_ms)

        self.pub_sub_manager.register_callback(callback, self.config.gui_bridge.metrics_topic, NodePerformance)

        self.register_refresh_function(named_value_plotter.refresh)

        return named_value_plotter


def check_all_fields_set(msg, msg_type) -> bool:
    unset_fields = set()
    for name in [field.name for field in msg_type.DESCRIPTOR.fields]:
        if not msg.HasField(name):
            unset_fields.add(name)

    if unset_fields:
        print(
            "The following fields are not set in the message:\n* {}".format(
                "\n* ".join(unset_fields)
            )
        )
        return False
    return True


def load_config() -> Config:
    config_path = Path(__file__).parent.parent / "config/config.pbtxt"
    with open(str(config_path), "r") as infile:
        data = infile.read()
        config = Parse(data, Config())
        print(config)
        check_all_fields_set(config, Config)
        return config


def shutdown(a, b):
    print("Caught signal, shutting down")
    QtCore.QCoreApplication.quit()


def main():

    signal.signal(signal.SIGINT, shutdown)
    signal.signal(signal.SIGTERM, shutdown)

    app = pg.mkQApp("Gui")
    w = RustwareGui()
    w.show()
    app.exec()


if __name__ == "__main__":
    main()
