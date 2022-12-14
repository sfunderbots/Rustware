# from pathlib import Path
# from enum import Enum
#
# SSL_VISION_MULTICAST_IP = "224.5.23.2"
# SSL_VISION_MULTICAST_PORT = 10020
#
# SSL_GAMECONTROLLER_MULTICAST_IP = "224.5.23.1"
# SSL_GAMECONTROLLER_MULTICAST_PORT = 10003
#
MILLIMETERS_PER_METER = 1000
METERS_PER_MILLIMETER = 1 / MILLIMETERS_PER_METER
#
# ZMQ_BASE_UNIX_SOCKET = "ipc:///tmp/underbots_zmq_"
#
# DIV_A_TOTAL_FIELD_X_LENGTH = 13.4
# DIV_A_TOTAL_FIELD_Y_LENGTH = 10.4
# DIV_B_TOTAL_FIELD_X_LENGTH = 10.4
# DIV_B_TOTAL_FIELD_Y_LENGTH = 7.4
#
ROBOT_MAX_RADIUS_METERS = 0.09
BALL_MAX_RADIUS_METERS = 0.0215
#
# MAX_ROBOT_ID = 15
# MAX_ROBOTS_DIV_B = 6
# MAX_ROBOTS_DIV_A = 11
#
# SSL_GAMECONTROLLER_BINARY_PATH = (
#     Path(__file__).parent.parent / "third_party/ssl_game_controller/binary"
# )
# SSL_GAMECONTROLLER_STATE_STORE_PATH = (
#     SSL_GAMECONTROLLER_BINARY_PATH.parent / "state-store.json.stream"
# )
#
# TIGERS_AUTOREF_PATH = Path(__file__).parent.parent / "third_party/tigers_autoref"
# # Assumes the binary has been prebuilt during installation
# TIGERS_AUTOREF_BINARY_PATH = TIGERS_AUTOREF_PATH / "autoReferee/bin/autoReferee"
#
# MAX_KICK_SPEED_METERS_PER_SECOND = 6.5
#
# FRIENDLY_ROBOT_MAX_SPEED_METERS_PER_SECOND = 2.5
# ENEMY_ROBOT_MAX_SPEED_METERS_PER_SECOND = 2.5
#
#
# class SslDivision(Enum):
#     DIV_A = 1
#     DIV_B = 2
#
#
# ERFORCE_SIMULATOR_GEOMETRY_CONFIG_PATH = (
#     Path(__file__).parent.parent / "third_party/erforce_simulator/config/simulator"
# )
# ERFORCE_SIMULATOR_DIVISION_A_CAMERA_CONFIG = (
#     ERFORCE_SIMULATOR_GEOMETRY_CONFIG_PATH / "2020.txt"
# )
# ERFORCE_SIMULATOR_DIVISION_B_CAMERA_CONFIG = (
#     ERFORCE_SIMULATOR_GEOMETRY_CONFIG_PATH / "2020B.txt"
# )
#
# ERFORCE_SIMULATOR_REALISM_CONFIG_PATH = (
#     Path(__file__).parent.parent
#     / "third_party/erforce_simulator/config/simulator-realism"
# )
# ERFORCE_SIMULATOR_REALISM_CONFIG_IDEAL = (
#     ERFORCE_SIMULATOR_REALISM_CONFIG_PATH / "None.txt"
# )
# ERFORCE_SIMULATOR_REALISM_CONFIG_REALISTIC = (
#     ERFORCE_SIMULATOR_REALISM_CONFIG_PATH / "RC2021.txt"
# )
